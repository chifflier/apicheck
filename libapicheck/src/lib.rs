#![feature(rustc_private)]

extern crate term;
extern crate json;

extern crate rustc;
extern crate rustc_plugin;
extern crate syntax;
extern crate syntax_pos;



use std::path::Path;
use std::rc::Rc;

use std::panic::{catch_unwind, AssertUnwindSafe};

use syntax::ast;
use syntax::parse::ParseSess;
use syntax::source_map::{SourceMap,FilePathMapping};
use syntax::source_map::edition::Edition;
use syntax::errors::{DiagnosticBuilder, Handler};
use syntax::errors::emitter::ColorConfig;

use std::fs::File;
use std::io;
use std::convert::From;

pub(crate) mod process;
pub(crate) mod items;
pub(crate) mod modules;

pub mod config;
use config::{Config,FileName};

use process::create_json_from_crate;
pub use items::check_item;

#[derive(Debug)]
pub enum ApiCheckError<'a> {
    ParseError(ParseError<'a>),
    IoError(io::Error),
}

impl<'a> From<io::Error> for ApiCheckError<'a> {
    fn from(e: io::Error) -> ApiCheckError<'a> {
        ApiCheckError::IoError(e)
    }
}

/// All the ways that parsing can fail.
#[derive(Debug)]
pub enum ParseError<'sess> {
    /// There was an error, but the parser recovered.
    Recovered,
    /// There was an error (supplied) and parsing failed.
    Error(DiagnosticBuilder<'sess>),
    /// The parser panicked.
    Panic,
}

pub fn process_file(input: String, config: &Config) {
    syntax::with_globals(Edition::from_session(), || process_file_inner(input, &config))
}

fn process_file_inner(input: String, config: &Config) {
    // build parsing session
    let codemap = Rc::new(SourceMap::new(FilePathMapping::empty()));
    let tty_handler = {
        let supports_color = term::stderr().map_or(false, |term| term.supports_color());
        let color_cfg = if supports_color {
            ColorConfig::Auto
        } else {
            ColorConfig::Never
        };
        Handler::with_tty_emitter(color_cfg, true, None, Some(codemap.clone()))
    };
    let mut parse_session = ParseSess::with_span_handler(tty_handler, codemap.clone());
    //
    let krate = match parse_input(input, &parse_session) {
        Ok(krate) => krate,
        Err(err) => {
            match err {
                ParseError::Error(mut diagnostic) => diagnostic.emit(),
                ParseError::Panic => {
                    // // Note that if you see this message and want more information,
                    // // then go to `parse_input` and run the parse function without
                    // // `catch_unwind` so rustfmt panics and you can get a backtrace.
                    // should_emit_verbose(&main_file, config, || {
                    //     println!("The Rust parser panicked")
                    // });
                }
                ParseError::Recovered => {}
            }
            // summary.add_parsing_error();
            // return Ok((summary, FileMap::new(), FormatReport::new()));
            panic!("parsing failed");
        }
    };

    let result = create_json_from_crate(&krate, &mut parse_session, &config);
    let json = result.expect("extracting JSON failed");
    write_json(&json, &config.output).expect("writing JSON failed");
}

fn parse_input<'sess>(file: String, parse_session: &'sess ParseSess) -> Result<ast::Crate, ParseError<'sess>> {
    //
    let file = Path::new(&file);
    let mut parser = syntax::parse::new_parser_from_file(&parse_session, &file);

    parser.cfg_mods = false;
    let mut parser = AssertUnwindSafe(parser);
    let result = catch_unwind(move || parser.0.parse_crate_mod());

    match result {
        Ok(Ok(c)) => {
            if parse_session.span_diagnostic.has_errors() {
                // Bail out if the parser recovered from an error.
                Err(ParseError::Recovered)
            } else {
                Ok(c)
            }
        }
        Ok(Err(e)) => Err(ParseError::Error(e)),
        Err(_) => Err(ParseError::Panic),
    }
}

fn write_json(js: &json::JsonValue, output: &FileName) -> Result<(),io::Error> {
    match &output {
        FileName::Stdin => panic!("Cannot output to stdin"),
        FileName::Stdout => println!("{}", js),
        FileName::Real(path) => {
            let mut file = File::create(path)?;
            let mut buf_writer = io::BufWriter::new(file);
            js.write(&mut buf_writer)?;
        },
    }
    Ok(())
}
