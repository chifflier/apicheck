extern crate ignore;
extern crate json;
extern crate term;
extern crate thiserror;

extern crate rustc_ast;
extern crate rustc_ast_pretty;
extern crate rustc_data_structures;
extern crate rustc_errors;
extern crate rustc_expand;
extern crate rustc_parse;
extern crate rustc_session;
extern crate rustc_span;

use std::path::Path;
use std::rc::Rc;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::fs::File;
use std::io;
use std::convert::From;

// use rustc_session::parse::ParseSess;
use rustc_ast::ast;
use rustc_span::source_map::{SourceMap,FilePathMapping};
use rustc_errors::{DiagnosticBuilder, Handler};
use rustc_errors::emitter::ColorConfig;

pub(crate) mod attr;
pub(crate) mod process;
mod input;
pub(crate) mod items;
pub(crate) mod modules;
pub(crate) mod result;
pub(crate) mod syntux;

pub mod config;
use config::{Config,FileName};

use process::create_json_from_crate;
pub use input::Input;
pub use items::check_item;
use result::OperationError;
use syntux::parser::{DirectoryOwnership, Parser, ParserError};
pub(crate) use syntux::session::ParseSess;

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

pub fn process_file(input: Input, config: &Config) -> Result<(), OperationError> {
    rustc_span::with_session_globals(config.edition, || process_project(input, &config))
}

fn process_project(input: Input, config: &Config) -> Result<(), OperationError> {

    let main_file = input.file_name();
    let input_is_stdin = main_file == FileName::Stdin;

    let mut parse_session = ParseSess::new(config)?;

    // Parse the crate.
    let recursive = true;
    let directory_ownership = input.to_directory_ownership(recursive);
    let original_snippet = if let Input::Text(ref str) = input {
        Some(str.to_owned())
    } else {
        None
    };

    let krate = match Parser::parse_crate(config, input, directory_ownership, &parse_session) {
        Ok(krate) => krate,
        Err(e) => {
            return Err(OperationError::ParseError {
                input: main_file,
                is_panic: e == ParserError::ParsePanicError,
            });
        }
    };

    let files = modules::ModResolver::new(
        &parse_session,
        directory_ownership.unwrap_or(DirectoryOwnership::UnownedViaMod),
        !input_is_stdin && recursive,
    )
    .visit_crate(&krate)?;

    // // build parsing session
    // let codemap = Rc::new(SourceMap::new(FilePathMapping::empty()));
    // let tty_handler = {
    //     let supports_color = term::stderr().map_or(false, |term| term.supports_color());
    //     let color_cfg = if supports_color {
    //         ColorConfig::Auto
    //     } else {
    //         ColorConfig::Never
    //     };
    //     Handler::with_tty_emitter(color_cfg, true, None, Some(codemap.clone()))
    // };
    // let mut parse_session = rustc_session::parse::ParseSess::with_span_handler(tty_handler, codemap.clone());
    // //
    // let krate = match parse_input(input, &parse_session) {
    //     Ok(krate) => krate,
    //     Err(err) => {
    //         match err {
    //             ParseError::Error(mut diagnostic) => diagnostic.emit(),
    //             ParseError::Panic => {
    //                 // // Note that if you see this message and want more information,
    //                 // // then go to `parse_input` and run the parse function without
    //                 // // `catch_unwind` so rustfmt panics and you can get a backtrace.
    //                 // should_emit_verbose(&main_file, config, || {
    //                 //     println!("The Rust parser panicked")
    //                 // });
    //             }
    //             ParseError::Recovered => {}
    //         }
    //         // summary.add_parsing_error();
    //         // return Ok((summary, FileMap::new(), FormatReport::new()));
    //         panic!("parsing failed");
    //     }
    // };

    let result = create_json_from_crate(&files, &mut parse_session, &config);
    let json = result.expect("extracting JSON failed");
    write_json(&json, &config.output).expect("writing JSON failed");
    Ok(())
}

fn parse_input<'sess>(file: String, parse_session: &'sess rustc_session::parse::ParseSess) -> Result<ast::Crate, ParseError<'sess>> {
    //
    let file = Path::new(&file);
    let mut parser = rustc_parse::new_parser_from_file(&parse_session, &file, None);

    // XXX parser.cfg_mods = false;
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
            let file = File::create(path)?;
            let mut buf_writer = io::BufWriter::new(file);
            js.write(&mut buf_writer)?;
        },
    }
    Ok(())
}
