//! tbf's one kind of failure: it stops, and its message begins `error:`.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pos {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Error {
    pub message: String,
    pub at: Option<Pos>,
}

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Error { message: message.into(), at: None }
    }

    pub fn at(message: impl Into<String>, at: Pos) -> Self {
        Error { message: message.into(), at: Some(at) }
    }

    /// The full message, naming the file when the error has a place in it.
    pub fn render(&self, path: &str) -> String {
        match self.at {
            Some(Pos { line, col }) => format!("error: {}\n  at {path}:{line}:{col}", self.message),
            None => format!("error: {}", self.message),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error: {}", self.message)
    }
}

impl std::error::Error for Error {}
