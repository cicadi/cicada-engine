use syn::{
    Attribute, Error, Result, Token, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

pub fn parse_attributes<T>(attrs: &[Attribute]) -> Result<Vec<T>>
where
    T: Parse,
{
    let mut out = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("vk") {
            let iter = attr
                .parse_args_with(Punctuated::<T, Token![,]>::parse_terminated)?
                .into_iter();
            out.extend(iter);
        }
    }

    Ok(out)
}

pub fn parse_eq<T: Parse>(input: ParseStream) -> Result<T> {
    if input.is_empty() {
        Err(Error::new(
            input.span(),
            "unexpected end of input, expected `= {value}`",
        ))
    } else {
        _ = input.parse::<Token![=]>()?;
        input.parse()
    }
}

pub fn parse_parenthesized<T: Parse>(input: ParseStream) -> Result<T> {
    let content;
    parenthesized!(content in input);
    content.parse()
}
