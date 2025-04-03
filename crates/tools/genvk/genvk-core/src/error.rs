#[derive(Debug)]
pub enum Error {
    Parse(genvk_parse::error::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(e) => e.fmt(f),
        }
    }
}

impl From<genvk_parse::error::Error> for Error {
    fn from(value: genvk_parse::error::Error) -> Self {
        Self::Parse(value)
    }
}
