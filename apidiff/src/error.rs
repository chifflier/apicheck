use std::io;
use std::str;
use json;

#[derive(Debug)]
pub enum ApiDiffError {
    IoError(io::Error),
    JsonError(json::Error),
    Utf8Error(str::Utf8Error),
}

impl From<io::Error> for ApiDiffError {
    fn from(e: io::Error) -> ApiDiffError {
        ApiDiffError::IoError(e)
    }
}

impl From<json::JsonError> for ApiDiffError {
    fn from(e: json::JsonError) -> ApiDiffError {
        ApiDiffError::JsonError(e)
    }
}

impl From<str::Utf8Error> for ApiDiffError {
    fn from(e: str::Utf8Error) -> ApiDiffError {
        ApiDiffError::Utf8Error(e)
    }
}


