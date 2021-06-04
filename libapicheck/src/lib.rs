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

use std::convert::From;
use std::fs::File;
use std::io;

use rustc_errors::DiagnosticBuilder;

pub(crate) mod attr;
pub(crate) mod ignore_path;
mod input;
pub(crate) mod items;
pub(crate) mod modules;
pub(crate) mod process;
pub(crate) mod syntux;

pub mod config;
pub mod context;
use crate::context::Context;
pub use config::{Config, FileName};
use modules::ModuleResolutionError;

pub use input::Input;
pub use items::check_item;
use process::create_json_from_crate;
use syntux::parser::{DirectoryOwnership, Parser};
pub(crate) use syntux::session::ParseSess;
use thiserror::Error;

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

/// The various errors that can occur during formatting. Note that not all of
/// these can currently be propagated to clients.
#[derive(Error, Debug)]
pub enum ErrorKind {
    /// Line has exceeded character limit (found, maximum).
    #[error(
        "line formatted, but exceeded maximum width \
         (maximum: {1} (see `max_width` option), found: {0})"
    )]
    LineOverflow(usize, usize),
    /// Line ends in whitespace.
    #[error("left behind trailing whitespace")]
    TrailingWhitespace,
    // /// TODO or FIXME item without an issue number.
    // #[error("found {0}")]
    // BadIssue(Issue),
    /// License check has failed.
    #[error("license check failed")]
    LicenseCheck,
    /// Used deprecated skip attribute.
    #[error("`rustfmt_skip` is deprecated; use `rustfmt::skip`")]
    DeprecatedAttr,
    /// Used a rustfmt:: attribute other than skip or skip::macros.
    #[error("invalid attribute")]
    BadAttr,
    /// An io error during reading or writing.
    #[error("io error: {0}")]
    IoError(io::Error),
    /// Error during module resolution.
    #[error("{0}")]
    ModuleResolutionError(#[from] ModuleResolutionError),
    /// Parse error occurred when parsing the input.
    #[error("parse error")]
    ParseError,
    /// The user mandated a version and the current version of Rustfmt does not
    /// satisfy that requirement.
    #[error("version mismatch")]
    VersionMismatch,
    /// If we had formatted the given node, then we would have lost a comment.
    #[error("not formatted because a comment would be lost")]
    LostComment,
    /// Invalid glob pattern in `ignore` configuration option.
    #[error("Invalid glob pattern found in ignore list: {0}")]
    InvalidGlobPattern(ignore::Error),
}

// impl ErrorKind {
//     fn is_comment(&self) -> bool {
//         match self {
//             ErrorKind::LostComment => true,
//             _ => false,
//         }
//     }
// }

impl From<io::Error> for ErrorKind {
    fn from(e: io::Error) -> ErrorKind {
        ErrorKind::IoError(e)
    }
}

pub fn process_file(input: Input, config: &Config) -> Result<(), ErrorKind> {
    rustc_span::with_session_globals(config.edition, || process_project(input, &config))
}

fn process_project(input: Input, config: &Config) -> Result<(), ErrorKind> {
    let main_file = input.file_name();
    let input_is_stdin = main_file == FileName::Stdin;

    let parse_session = ParseSess::new(config)?;

    // Parse the crate.
    let recursive = true;
    let directory_ownership = input.to_directory_ownership(recursive);
    let _original_snippet = if let Input::Text(ref str) = input {
        Some(str.to_owned())
    } else {
        None
    };

    let krate = match Parser::parse_crate(input, &parse_session) {
        Ok(krate) => krate,
        Err(e) => {
            eprintln!("Parse error:\n{:?}", e);
            return Err(ErrorKind::ParseError);
        }
    };

    let files = modules::ModResolver::new(
        &parse_session,
        directory_ownership.unwrap_or(DirectoryOwnership::UnownedViaBlock),
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

    let context = Context::new(config);

    let result = create_json_from_crate(&files, &context);
    let json = result.expect("extracting JSON failed");
    write_json(&json, &config.output).expect("writing JSON failed");
    Ok(())
}

fn write_json(js: &json::JsonValue, output: &FileName) -> Result<(), io::Error> {
    match &output {
        FileName::Stdin => panic!("Cannot output to stdin"),
        FileName::Stdout => println!("{}", js),
        FileName::Real(path) => {
            let file = File::create(path)?;
            let mut buf_writer = io::BufWriter::new(file);
            js.write(&mut buf_writer)?;
        }
    }
    Ok(())
}
