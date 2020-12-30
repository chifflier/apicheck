use crate::modules::DirectoryOwnership;
use crate::FileName;
use rustc_span::symbol;
use std::path::PathBuf;

/// The input to rustfmt.
#[derive(Debug)]
pub enum Input {
    /// A file on the filesystem.
    File(PathBuf),
    /// A UTF-8 string, in many cases from stdin.
    Text(String),
}

impl Input {
    pub(crate) fn file_name(&self) -> FileName {
        match *self {
            Input::File(ref file) => FileName::Real(file.clone()),
            Input::Text(..) => FileName::Stdin,
        }
    }

    pub(crate) fn to_directory_ownership(&self, recursive: bool) -> Option<DirectoryOwnership> {
        match self {
            // On recursive mode, we assume that input is the root file.
            Input::File(..) if recursive => None,
            Input::File(ref file) => {
                // If there exists a directory with the same name as an input,
                // then the input should be parsed as a sub module.
                let file_stem = file.file_stem()?;
                if file.parent()?.to_path_buf().join(file_stem).is_dir() {
                    Some(DirectoryOwnership::Owned {
                        relative: file_stem.to_str().map(symbol::Ident::from_str),
                    })
                } else {
                    None
                }
            }
            Input::Text(..) => None,
        }
    }
}
