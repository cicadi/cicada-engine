use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DataEnum, DataStruct, DeriveInput, Error, Ident};

pub fn expand(input: DeriveInput) -> Result<TokenStream, Error> {
    match input.data {
        Data::Struct(data) => expand_struct(input.ident, input.attrs, data),
        Data::Enum(data) => expand_enum(input.ident, input.attrs, data),
        Data::Union(data) => Err(Error::new(
            data.union_token.span,
            "`#[derive(FromXml)]` not supported for `union` types",
        )),
    }
}

fn expand_struct(
    _ident: Ident,
    _attrs: Vec<Attribute>,
    _data: DataStruct,
) -> Result<TokenStream, Error> {
    Ok(quote!())
}

fn expand_enum(
    _ident: Ident,
    _attrs: Vec<Attribute>,
    _data: DataEnum,
) -> Result<TokenStream, Error> {
    Ok(quote!())
}
