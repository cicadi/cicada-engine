use std::path::Path;

use genvk_core::{
    Vulkan,
    error::Error,
    types::{Type, TypeEnum, TypeKind},
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::write_binding_file;

pub(crate) fn generate_enums_file(root: &Path, vk: &Vulkan) -> Result<(), Error> {
    write_binding_file(root.join("enums.rs"), generate_enums_file_content(vk)?)
}

fn generate_enums_file_content(vk: &Vulkan) -> Result<TokenStream, Error> {
    let mut contents = TokenStream::new();
    contents.extend(generate_enums_file_preamble(vk)?);

    for id in vk.types.enum_type_ids.iter() {
        match vk.types.id_to_type.get(id) {
            Some(Type {
                api_name: name,
                kind: TypeKind::Enum(item),
                ..
            }) => contents.extend(generate_enum_content(vk, &name, &item)?),
            _ => return Err(Error::invalid_state("")),
        }
    }

    Ok(contents)
}

fn generate_enums_file_preamble(_vk: &Vulkan) -> Result<TokenStream, Error> {
    Ok(quote!())
}

fn generate_enum_content(_vk: &Vulkan, _name: &str, item: &TypeEnum) -> Result<TokenStream, Error> {
    let mut contents = TokenStream::new();

    let name_ident = Ident::new(&item.output_name, Span::call_site());
    contents.extend(quote!(
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
        #[repr(transparent)]
        pub struct #name_ident(pub(crate) i32);
    ));

    Ok(contents)
}
