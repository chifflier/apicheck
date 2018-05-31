use std::default::Default;
use std::fmt;
use std::path::PathBuf;

pub struct Config {
    pub debug: usize,

    pub output: FileName,
}

/// Defines the name of an input - either a file or stdin.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum FileName {
    Real(PathBuf),
    Stdin,
    Stdout,
}

impl Default for Config {
    fn default() -> Config {
        Config{
            debug: 0,
            output: FileName::Stdout,
        }
    }
}

impl fmt::Display for FileName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileName::Real(ref p) => write!(f, "{}", p.to_str().unwrap_or(&"<invalid path>")),
            FileName::Stdin       => write!(f, "<stdin>"),
            FileName::Stdout      => write!(f, "<stdout>"),
        }
    }
}
