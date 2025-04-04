use crate::parse;

#[derive(Debug)]
pub enum Error {
    Parse(crate::parse::Error),

    DuplicateType,
    PopulateLoop,

    InvalidState(String),
    OutSyn(syn::Error),

    Io(std::io::Error),
}

impl Error {
    pub fn invalid_state<S: AsRef<str>>(msg: S) -> Self {
        Self::InvalidState(msg.as_ref().to_string())
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(e) => e.fmt(f),

            Error::DuplicateType => write!(f, "duplicate type being registered"),
            Error::PopulateLoop => write!(f, "population ran into infinite loop"),

            Error::InvalidState(msg) => write!(f, "{msg}"),
            Error::OutSyn(e) => write!(f, "error while generating output: {e}"),

            Error::Io(e) => e.fmt(f),
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
