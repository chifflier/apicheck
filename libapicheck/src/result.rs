use crate::config::FileName;
use crate::modules::ModuleResolutionError;
use thiserror::Error;

/// Represent errors unrelated to formatting issues.
#[derive(Error, Debug)]
pub enum OperationError {
    /// The user mandated a version and the current version of rustfmt does not
    /// satisfy that requirement.
    #[error("version mismatch")]
    VersionMismatch,
    /// Error during module resolution.
    #[error("{0}")]
    ModuleResolutionError(#[from] ModuleResolutionError),
    /// Invalid glob pattern in `ignore` configuration option.
    #[error("invalid glob pattern found in ignore list: {0}")]
    InvalidGlobPattern(ignore::Error),
    /// Parse error occurred while parsing the input.
    #[error("failed to parse {input}")]
    ParseError { input: FileName, is_panic: bool },
    /// Io error.
    #[error("{0}")]
    IoError(#[from] std::io::Error),
}
