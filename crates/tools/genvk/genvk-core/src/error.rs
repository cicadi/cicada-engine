use crate::parse;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),

    Parse(crate::parse::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => e.fmt(f),

            Error::Parse(e) => e.fmt(f),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<crate::parse::Error> for Error {
    fn from(value: parse::Error) -> Self {
        Self::Parse(value)
    }
}
