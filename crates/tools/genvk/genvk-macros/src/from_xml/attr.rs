use syn::{
    Error, Ident, LitStr, Result, Type,
    parse::{Parse, ParseStream},
};

use super::util::{parse_eq, parse_parenthesized};

pub enum StructAttr {
    Tag(LitStr),
}

impl Parse for StructAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let name_str = name.to_string();

        Ok(match name_str.as_str() {
            "tag" => Self::Tag(parse_eq(input)?),
            _ => {
                return Err(Error::new(
                    name.span(),
                    format!("invalid attribute `{name_str}`"),
                ));
            }
        })
    }
}

pub enum FieldAttr {
    Name(Ident, LitStr),
    Optional(Ident),
    Items(Ident, Type),
    Text(Ident),
    Generic(Ident),
}

impl Parse for FieldAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let name_str = name.to_string();

        Ok(match name_str.as_str() {
            "name" => Self::Name(name, parse_eq(input)?),
            "optional" => Self::Optional(name),
            "items" => Self::Items(name, parse_parenthesized(input)?),
            "text" => Self::Text(name),
            "generic" => Self::Generic(name),
            _ => {
                return Err(Error::new(
                    name.span(),
                    format!("invalid attribute `{name_str}`"),
                ));
            }
        })
    }
}

pub enum EnumAttr {
    Tagged(Ident),
    Var(Ident),
    Eval(Ident),

    Tag(Ident, LitStr),
    Attr(Ident, LitStr),
}

impl Parse for EnumAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let name_str = name.to_string();

        Ok(match name_str.as_str() {
            "tagged" => Self::Tagged(name),
            "var" => Self::Var(name),
            "eval" => Self::Eval(name),

            "tag" => Self::Tag(name, parse_eq(input)?),
            "attr" => Self::Attr(name, parse_eq(input)?),
            _ => {
                return Err(Error::new(
                    name.span(),
                    format!("invalid attribute `{name_str}`"),
                ));
            }
        })
    }
}

pub enum VariantAttr {
    Attr(Ident, LitStr),
    Value(Ident, LitStr),
    None(Ident),
}

impl Parse for VariantAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let name_str = name.to_string();

        Ok(match name_str.as_str() {
            "attr" => Self::Attr(name, parse_eq(input)?),
            "value" => Self::Value(name, parse_eq(input)?),
            "none" => Self::None(name),
            _ => {
                return Err(Error::new(
                    name.span(),
                    format!("invalid attribute `{name_str}`"),
                ));
            }
        })
    }
}
