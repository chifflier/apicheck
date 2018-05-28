use std::default::Default;
use std::fmt;
use std::path::PathBuf;

pub struct Config {
    pub debug: usize,
}

/// Defines the name of an input - either a file or stdin.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum FileName {
    Real(PathBuf),
    Stdin,
}

impl Default for Config {
    fn default() -> Config {
        Config{
            debug: 0,
        }
    }
}

impl fmt::Display for FileName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileName::Real(ref p) => write!(f, "{:?}", p),
            FileName::Stdin       => write!(f, "<stdin>"),
        }
    }
}
