use std::collections::HashMap;

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

    UnexpectedTagStart {
        parent: String,
        tag: String,
    },
    UnexpectedTagEnd {
        parent: String,
        tag: String,
    },

    BadTaggedVariant {
        name: String,
        tag: String,
    },
    BadVarVariant {
        name: String,
    },
    BadEvalVariant {
        name: String,
        tag: String,
        attr: String,
        value: Option<String>,
    },

    ExtraAttrs {
        tag: String,
        attrs: HashMap<String, String>,
    },
    MissingAttr {
        tag: String,
        attr: String,
    },
    BadAttr {
        tag: String,
        attr: String,
        value: String,
    },

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

            ErrorKind::UnexpectedTagStart { parent, tag } => {
                write!(f, "unexpected start tag <{tag}> inside tag <{parent}>")
            }
            ErrorKind::UnexpectedTagEnd { parent, tag } => {
                write!(f, "unexpected end tag </{tag}> inside tag <{parent}>")
            }

            ErrorKind::BadTaggedVariant { name, tag } => {
                write!(f, "bad tagged variant `<{tag}>` for enum `{name}`")
            }
            ErrorKind::BadVarVariant { name } => write!(f, "bad variant for enum `{name}`"),
            ErrorKind::BadEvalVariant {
                name,
                tag,
                attr,
                value,
            } => {
                write!(
                    f,
                    "bad eval variant `<{tag}>` with `{attr} = {value}` for enum `{name}`",
                    value = value.as_ref().map(String::as_str).unwrap_or_else(|| "None")
                )
            }

            ErrorKind::ExtraAttrs { tag, attrs } => write!(
                f,
                "tag `<{tag}>` has additional unprocessed attributes: {attrs:?}"
            ),
            ErrorKind::MissingAttr { tag, attr } => {
                write!(f, "missing  attr `{attr}` for tag `<{tag}>`")
            }
            ErrorKind::BadAttr { tag, attr, value } => {
                write!(f, "tag <{tag}> has invalid attr `{attr} = {value}`")
            }

            ErrorKind::XmlRead(e) => e.fmt(f),
        }
    }
}
