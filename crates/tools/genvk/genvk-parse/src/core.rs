use std::{borrow::Cow, collections::HashMap, io::Read};

use xml::{EventReader, common::Position};

use crate::{
    error::{Error, ErrorKind},
    util::Extract,
};

pub trait FromStr: Sized {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String>;
}

pub trait ParseAttr {
    type Output;

    fn parse_attr<R: Read>(
        reader: &EventReader<R>,
        tag: &str,
        attr: &str,
        attrs: &mut HashMap<String, String>,
    ) -> Result<Option<Self::Output>, Error>;
}

impl<T> ParseAttr for T
where
    T: FromStr,
{
    type Output = T;

    fn parse_attr<R: Read>(
        reader: &EventReader<R>,
        tag: &str,
        attr: &str,
        attrs: &mut HashMap<String, String>,
    ) -> Result<Option<T>, Error> {
        Ok(match attrs.remove(attr) {
            Some(value) => Some(T::from_str(Cow::Owned(value)).map_err(|value| {
                Error::new(
                    reader.position(),
                    ErrorKind::BadAttr {
                        tag: tag.to_string(),
                        attr: attr.to_string(),
                        value: value.to_string(),
                    },
                )
            })?),
            None => None,
        })
    }
}

impl<T> ParseAttr for Vec<T>
where
    T: FromStr,
{
    type Output = Vec<T>;

    fn parse_attr<R: Read>(
        reader: &EventReader<R>,
        tag: &str,
        attr: &str,
        attrs: &mut HashMap<String, String>,
    ) -> Result<Option<Vec<T>>, Error> {
        Ok(match attrs.remove(attr) {
            Some(value) => Some(
                value
                    .split(',')
                    .map(|s| T::from_str(Cow::Borrowed(s)))
                    .collect::<Result<_, _>>()
                    .map_err(|_| {
                        Error::new(
                            reader.position(),
                            ErrorKind::BadAttr {
                                tag: tag.to_string(),
                                attr: attr.to_string(),
                                value,
                            },
                        )
                    })?,
            ),
            None => None,
        })
    }
}

impl FromStr for bool {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        <bool as std::str::FromStr>::from_str(s.as_ref()).map_err(|_| s.into_owned())
    }
}

impl FromStr for i32 {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        <i32 as std::str::FromStr>::from_str(s.as_ref()).map_err(|_| s.into_owned())
    }
}

impl FromStr for u32 {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        <u32 as std::str::FromStr>::from_str(s.as_ref()).map_err(|_| s.into_owned())
    }
}

impl FromStr for String {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        Ok(s.into_owned())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Api {
    Disabled,
    Vulkan,
    VulkanSc,
}

impl Extract for Api {
    type Output = Api;
}

impl FromStr for Api {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        Ok(match s.as_ref() {
            "disabled" => Self::Disabled,
            "vulkan" => Self::Vulkan,
            "vulkansc" => Self::VulkanSc,
            _ => return Err(s.to_string()),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Deprecation {
    True = 1,
    Aliased,
    Ignored,
}

impl Extract for Deprecation {
    type Output = Deprecation;
}

impl FromStr for Deprecation {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        Ok(match s.as_ref() {
            "true" => Self::True,
            "aliased" => Self::Aliased,
            "ignored" => Self::Ignored,
            _ => return Err(s.to_string()),
        })
    }
}

#[derive(Debug)]
pub enum Len {
    Member(String),
    Math { value: String, alt: String },
    NullTerminated, // null-terminated
    Ptr,            // 1
}

impl Extract for Len {
    type Output = Len;
}

impl ParseAttr for Len {
    type Output = Self;

    fn parse_attr<R: Read>(
        reader: &EventReader<R>,
        tag: &str,
        attr: &str,
        attrs: &mut HashMap<String, String>,
    ) -> Result<Option<Self>, Error> {
        Ok(match attrs.remove(attr) {
            Some(value) => Some(match value.as_str() {
                "null-terminated" => Self::NullTerminated,
                "1" => Self::Ptr,
                s if s.starts_with("latexmath:") => Self::Math {
                    value,
                    alt: attrs.remove("altlen").ok_or(Error::new(
                        reader.position(),
                        ErrorKind::MissingAttr {
                            tag: tag.to_string(),
                            attr: "altlen".to_string(),
                        },
                    ))?,
                },
                _ => Self::Member(value),
            }),
            None => None,
        })
    }
}

impl ParseAttr for Vec<Len> {
    type Output = Vec<Len>;

    fn parse_attr<R: Read>(
        reader: &EventReader<R>,
        tag: &str,
        attr: &str,
        attrs: &mut HashMap<String, String>,
    ) -> Result<Option<Self::Output>, Error> {
        if attrs.contains_key("altlen") {
            Ok(Len::parse_attr(reader, tag, attr, attrs)?.map(|len| vec![len]))
        } else {
            Ok(attrs.remove(attr).map(|value| {
                value
                    .split(',')
                    .map(|s| match s {
                        "null-terminated" => Len::NullTerminated,
                        "1" => Len::Ptr,
                        _ => Len::Member(s.to_string()),
                    })
                    .collect()
            }))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitType {
    Min,
    Max,
    PoT,
    Mul,
    Bits,
    Bitmask,
    Range,
    Struct,
    Exact,
    NoAuto,
}

impl Extract for LimitType {
    type Output = LimitType;
}

impl FromStr for LimitType {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        Ok(match s.as_ref() {
            "min" => Self::Min,
            "max" => Self::Max,
            "pot" => Self::PoT,
            "mul" => Self::Mul,
            "bits" => Self::Bits,
            "bitmask" => Self::Bitmask,
            "range" => Self::Range,
            "struct" => Self::Struct,
            "exact" => Self::Exact,
            "noauto" => Self::NoAuto,
            _ => return Err(s.into_owned()),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Queue {
    Compute,
    Decode,
    Encode,
    Graphics,
    Transfer,
    SparseBinding,
    OpticalFlow,
}

impl Extract for Queue {
    type Output = Queue;
}

impl FromStr for Queue {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        Ok(match s.as_ref() {
            "compute" => Self::Compute,
            "decode" => Self::Decode,
            "encode" => Self::Encode,
            "graphics" => Self::Graphics,
            "transfer" => Self::Transfer,
            "sparse_binding" => Self::SparseBinding,
            "opticalflow" => Self::OpticalFlow,
            _ => return Err(s.into_owned()),
        })
    }
}

#[derive(Debug)]
pub enum Depends {
    Feature(String),
    Group(Box<Depends>),
    And(Box<Depends>, Box<Depends>),
    Or(Box<Depends>, Box<Depends>),
}

impl Extract for Depends {
    type Output = Depends;
}

impl FromStr for Depends {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        Depends::parse(s.as_ref()).map_err(|_| s.into_owned())
    }
}

impl Depends {
    fn parse(s: &str) -> Result<Self, ParseDependsError> {
        let _a = Self::parse_recursive(s, 0)?.0.ok_or(ParseDependsError);
        _a
    }

    fn parse_recursive(s: &str, pos: usize) -> Result<(Option<Self>, usize), ParseDependsError> {
        let (Some(depends), pos) = Self::parse_primitive(s, pos)? else {
            return Ok((None, pos));
        };

        let mut chars = s[pos..].chars();
        match chars.next() {
            Some(ch) => Ok({
                let left = depends;
                let (Some(right), pos) = Self::parse_recursive(s, pos + ch.len_utf8())? else {
                    return Ok((Some(left), pos));
                };

                (
                    Some(match ch {
                        '+' => Self::And(Box::new(left), Box::new(right)),
                        ',' => Self::Or(Box::new(left), Box::new(right)),
                        _ => return Err(ParseDependsError),
                    }),
                    pos,
                )
            }),
            None => Ok((Some(depends), pos)),
        }
    }

    fn parse_primitive(s: &str, pos: usize) -> Result<(Option<Self>, usize), ParseDependsError> {
        let mut chars = s[pos..].chars();
        match chars.next() {
            Some(ch) => match ch {
                '(' => Self::parse_group(s, pos),
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | ':' => Self::parse_feature(s, pos),
                _ => Ok((None, pos)),
            },
            None => Ok((None, pos)),
        }
    }

    fn parse_feature(s: &str, mut pos: usize) -> Result<(Option<Self>, usize), ParseDependsError> {
        let start = pos;
        let mut chars = s[pos..].chars();
        while let Some(ch) = chars.next() {
            match ch {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | ':' => pos += ch.len_utf8(),
                _ => break,
            }
        }

        Ok((
            if start == pos {
                None
            } else {
                Some(Self::Feature(s[start..pos].to_string()))
            },
            pos,
        ))
    }

    fn parse_group(s: &str, pos: usize) -> Result<(Option<Self>, usize), ParseDependsError> {
        assert_eq!(s[pos..].chars().next(), Some('('));
        let (Some(depends), pos) = Self::parse_recursive(s, pos + '('.len_utf8())? else {
            return Err(ParseDependsError);
        };

        if s[pos..].chars().next() == Some(')') {
            Ok((Some(Self::Group(Box::new(depends))), pos + ')'.len_utf8()))
        } else {
            Err(ParseDependsError)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseDependsError;

impl std::error::Error for ParseDependsError {}

impl std::fmt::Display for ParseDependsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "depends failed parsing")
    }
}
