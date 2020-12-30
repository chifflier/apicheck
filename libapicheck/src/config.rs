use std::collections::{hash_set, HashSet};
use std::default::Default;
use std::fmt;
use std::path::{Path, PathBuf};
use rustc_span::source_map::edition::Edition;

pub struct Config {
    pub debug: usize,

    pub edition: Edition,

    pub output: FileName,

    hide_parse_errors: bool,
}

/// A set of directories, files and modules that rustfmt should ignore.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct IgnoreList {
    /// A set of path specified in rustfmt.toml.
    path_set: HashSet<PathBuf>,
    /// A path to rustfmt.toml.
    rustfmt_toml_path: PathBuf,
}

impl IgnoreList {
    pub fn add_prefix(&mut self, dir: &Path) {
        self.rustfmt_toml_path = dir.to_path_buf();
    }

    pub fn rustfmt_toml_path(&self) -> &Path {
        &self.rustfmt_toml_path
    }
}

impl<'a> IntoIterator for &'a IgnoreList {
    type Item = &'a PathBuf;
    type IntoIter = hash_set::Iter<'a, PathBuf>;

    fn into_iter(self) -> Self::IntoIter {
        self.path_set.iter()
    }
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
            edition: Edition::Edition2018,
            output: FileName::Stdout,
            hide_parse_errors: false,
        }
    }
}

impl Config {
    pub fn hide_parse_errors(&self) -> bool {
        self.hide_parse_errors
    }

    pub fn ignore(&self) -> IgnoreList {
        IgnoreList::default()
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
