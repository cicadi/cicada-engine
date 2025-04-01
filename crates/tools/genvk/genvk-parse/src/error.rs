use xml::common::{Position, TextPosition};

#[derive(Debug)]
pub struct Error {
    pub pos: TextPosition,
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    UnexpectedDocStart,
    UnexpectedDocEnd,

    MissingAttr { tag: String, attr: String },
    BadAttr { tag: String, attr: String, value: String },

    XmlRead(xml::reader::Error),
}

impl Error {
    pub fn new(pos: TextPosition, kind: ErrorKind) -> Self {
        Self { pos, kind }
    }
}

impl From<xml::reader::Error> for Error {
    fn from(value: xml::reader::Error) -> Self {
        Self::new(value.position(), ErrorKind::XmlRead(value))
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: ", self.pos)?;
        match &self.kind {
            ErrorKind::UnexpectedDocStart => write!(f, "unexpected doc start"),
            ErrorKind::UnexpectedDocEnd => write!(f, "unexpected doc end"),

            ErrorKind::MissingAttr { tag, attr } => {
                write!(f, "missing  attr `{attr}` for tag <{tag}>")
            }
            ErrorKind::BadAttr { tag, attr, value } => {
                write!(f, "tag <{tag}> has invalid attr `{attr} = {value}`")
            }

            ErrorKind::XmlRead(e) => e.fmt(f),
        }
    }
}
