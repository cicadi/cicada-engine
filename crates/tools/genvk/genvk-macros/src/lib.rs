mod from_xml;

use proc_macro::TokenStream;
use syn::{DeriveInput, Error, parse_macro_input};

#[proc_macro_derive(FromXml, attributes(vk))]
pub fn derive_from_xml(input: TokenStream) -> TokenStream {
    from_xml::expand(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
