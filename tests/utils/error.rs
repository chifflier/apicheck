use std::convert::From;

pub type Result<T> = std::result::Result<T, TestError>;

#[derive(Debug)]
pub enum TestError {
    EnvError(std::env::VarError),
    IOError(std::io::Error),
    JSonError(json::Error),
}

impl From<std::env::VarError> for TestError {
    fn from(error: std::env::VarError) -> Self {
        TestError::EnvError(error)
    }
}

impl From<std::io::Error> for TestError {
    fn from(error: std::io::Error) -> Self {
        TestError::IOError(error)
    }
}

impl From<json::Error> for TestError {
    fn from(error: json::Error) -> Self {
        TestError::JSonError(error)
    }
}
