use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, LitStr, Result, Type};

pub struct Field {
    pub name: Ident,
    pub attr: LitStr,
    pub ty: Type,

    pub optional: bool,
}

pub struct Items {
    pub name: Ident,
    pub ty: Type,
}

pub struct Text {
    pub name: Ident,
    pub generic: bool,
}

pub struct StructModel {
    pub name: Ident,
    pub tag: LitStr,

    pub fields: Vec<Field>,
    pub items: Option<Items>,
    pub text: Option<Text>,
}

impl StructModel {
    pub fn new(
        name: Ident,
        tag: LitStr,
        fields: Vec<Field>,
        items: Option<Items>,
        text: Option<Text>,
    ) -> Self {
        Self {
            name,
            tag,
            fields,
            items,
            text,
        }
    }

    pub fn expand(self) -> Result<TokenStream> {
        let parse_attributes_fn_body = self.parse_attributes_fn_body()?;
        let parse_item_fn_body = self.parse_item_fn_body()?;
        let parse_text_fn_body = self.parse_text_fn_body()?;

        let Self { name, tag, .. } = self;
        Ok(quote!(
            impl #name {
                const TAG: &'static str = #tag;

                fn parse_attributes<R: std::io::Read>(
                    &mut self,
                    reader: &mut xml::EventReader<R>,
                    mut attrs: HashMap<String, String>,
                ) -> Result<(), crate::error::Error> {
                    #parse_attributes_fn_body
                }

                fn parse_item<R: std::io::Read>(
                    &mut self,
                    reader: &mut xml::EventReader<R>,
                    tag: String,
                    attrs: HashMap<String, String>,
                ) -> Result<(), crate::error::Error> {
                    #parse_item_fn_body
                }

                fn parse_text<R: std::io::Read>(
                    &mut self,
                    reader: &mut xml::EventReader<R>,
                    text: String,
                ) -> Result<(), crate::error::Error> {
                    #parse_text_fn_body
                }

                fn parse_misc<R: std::io::Read>(
                    &mut self,
                    reader: &mut xml::EventReader<R>,
                    event: xml::reader::XmlEvent,
                ) -> Result<(), crate::error::Error> {
                    Ok(())
                }
            }

            impl crate::FromXml for #name {
                fn from_xml<R: std::io::Read>(
                    reader: &mut xml::EventReader<R>,
                    tag: String,
                    attrs: std::collections::HashMap<String, String>,
                ) -> Result<Self, crate::error::Error> {
                    let mut out = <Self as Default>::default();

                    loop {
                        match reader.next()? {
                            xml::reader::XmlEvent::StartElement {
                                name, attributes, ..
                            } => out.parse_item(reader, name.local_name, attributes.into_hash_map())?,
                            xml::reader::XmlEvent::EndElement { name } => {
                                if name.local_name == Self::TAG {
                                    break;
                                } else {
                                    return Err(crate::error::Error::new(
                                        reader.position(),
                                        crate::error::ErrorKind::UnexpectedTagEnd {
                                            parent: Self::TAG.to_string(),
                                            tag: name.local_name,
                                        },
                                    ));
                                }
                            }

                            xml::reader::XmlEvent::Characters(text)
                            | xml::reader::XmlEvent::Whitespace(text) => out.parse_text(reader, text)?,

                            xml::reader::XmlEvent::StartDocument { .. } => {
                                return Err(crate::error::Error::new(
                                    xml::common::Position::position(reader),
                                    crate::error::ErrorKind::UnexpectedDocStart,
                                ));
                            }
                            xml::reader::XmlEvent::EndDocument => {
                                return Err(crate::error::Error::new(
                                    xml::common::Position::position(reader),
                                    crate::error::ErrorKind::UnexpectedDocEnd,
                                ));
                            }

                            event => out.parse_misc(reader, event)?,
                        }
                    }

                    Ok(out)
                }
            }
        ))
    }

    fn parse_attributes_fn_body(&self) -> Result<TokenStream> {
        let mut body = TokenStream::new();
        for field in self.fields.iter() {
            let Field {
                name,
                attr: attr_name,
                ty,
                optional,
            } = field;

            let option_unwrap = if *optional {
                quote!()
            } else {
                quote!(.ok_or(crate::error::Error::new(
                    xml::common::Position::position(reader),
                    crate::error::ErrorKind::MissingAttr {
                        tag: Self::TAG.to_string(),
                        attr: #attr_name.to_string(),
                    }
                ))?)
            };

            body.extend(quote!(
                self.#name =
                    <<#ty as crate::util::Extract>::Output as crate::core::ParseAttr>::parse_attr(
                        reader, Self::TAG, #attr_name, &mut attrs
                    )?#option_unwrap;
            ));
        }

        Ok(quote!(
            #body
            if !attrs.is_empty() {
                Err(crate::error::Error::new(
                    xml::common::Position::position(reader),
                    crate::error::ErrorKind::UnexpectedTagEnd {
                        parent: Self::TAG.to_string(),
                        tag: Self::TAG.to_owned(),
                    },
                ))
            } else {
                Ok(())
            }
        ))
    }

    fn parse_item_fn_body(&self) -> Result<TokenStream> {
        Ok(match &self.items {
            Some(items) => {
                let Items { name, ty } = items;

                quote!(
                    self.#name.push(<#ty as crate::FromXml>::from_xml(reader, tag, attrs)?);
                    Ok(())
                )
            }
            None => quote!(Ok(())),
        })
    }

    fn parse_text_fn_body(&self) -> Result<TokenStream> {
        Ok(match &self.text {
            Some(text) => {
                let Text { name, generic } = text;

                if *generic {
                    quote!(
                        self.#name.push(GenericItem::String(text));
                        Ok(())
                    )
                } else {
                    quote!(
                        self.#name.push_str(&text);
                        Ok(())
                    )
                }
            }
            None => quote!(Ok(())),
        })
    }
}

pub struct TaggedVariant {
    pub name: Ident,
    pub ty: Type,
}

pub struct TaggedEnumModel {
    pub name: Ident,
    pub variants: Vec<TaggedVariant>,
}

impl TaggedEnumModel {
    pub fn new(name: Ident, variants: Vec<TaggedVariant>) -> Self {
        Self { name, variants }
    }

    pub fn expand(self) -> Result<TokenStream> {
        let Self { name, variants, .. } = self;
        let name_str = name.to_string();

        let mut match_body = TokenStream::new();
        for variant in variants {
            let TaggedVariant { name, ty } = variant;
            match_body.extend(quote!(
                #ty::TAG => Self::#name(<#ty as crate::FromXml>::from_xml(reader, tag, attrs)?),
            ));
        }

        match_body.extend(quote!(
            _ => {
                return Err(crate::error::Error::new(
                    xml::common::Position::position(reader),
                    crate::error::ErrorKind::BadTaggedVariant { name: #name_str.to_string(), tag },
                ))
            },
        ));

        Ok(quote!(
            impl crate::FromXml for #name {
                fn from_xml<R: std::io::Read>(
                    reader: &mut xml::EventReader<R>,
                    tag: String,
                    attrs: std::collections::HashMap<String, String>,
                ) -> Result<Self, crate::error::Error> {
                    Ok(match tag.as_str() {
                        #match_body
                    })
                }
            }
        ))
    }
}

pub struct VarVariant {
    pub name: Ident,
    pub ty: Type,
    pub attr: LitStr,
}

pub struct NoneVariant {
    pub name: Ident,
    pub ty: Type,
}

pub struct VarEnumModel {
    pub name: Ident,
    pub tag: LitStr,

    pub variants: Vec<VarVariant>,
    pub none: Option<NoneVariant>,
}

impl VarEnumModel {
    pub fn new(
        name: Ident,
        tag: LitStr,
        variants: Vec<VarVariant>,
        none: Option<NoneVariant>,
    ) -> Self {
        Self {
            name,
            tag,
            variants,
            none,
        }
    }

    pub fn expand(self) -> Result<TokenStream> {
        let Self {
            name,
            tag,
            variants,
            none,
            ..
        } = self;
        let name_str = name.to_string();

        let mut first = true;
        let mut body = TokenStream::new();
        for variant in variants {
            let VarVariant { name, ty, attr } = variant;

            let condition = quote!(attrs.contains_key(#attr));
            let if_variant = quote!(Ok(Self::#name(#ty::from_xml(reader, tag, attrs)?)));
            if first {
                body.extend(quote!(if #condition {
                    #if_variant
                }));
            } else {
                body.extend(quote!( else if #condition {
                    #if_variant
                }));
            }

            first = false;
        }

        let none_variant = if let Some(NoneVariant { name, ty }) = none {
            quote!(Ok(Self::#name(#ty::from_xml(reader, tag, attrs)?)))
        } else {
            quote!(Err(crate::error::Error::new(
                xml::common::Position::position(reader),
                crate::error::ErrorKind::BadVarVariant {
                    name: #name_str.to_string(),
                }
            )))
        };

        if first {
            body.extend(none_variant);
        } else {
            body.extend(quote!(else {
                #none_variant
            }))
        }

        Ok(quote!(
            impl #name {
                pub const TAG: &'static str = #tag;
            }

            impl crate::FromXml for #name {
                fn from_xml<R: std::io::Read>(
                    reader: &mut xml::EventReader<R>,
                    tag: String,
                    attrs: std::collections::HashMap<String, String>,
                ) -> Result<Self, crate::error::Error> {
                    #body
                }
            }
        ))
    }
}

pub struct EvalVariant {
    pub name: Ident,
    pub ty: Type,
    pub value: LitStr,
}

pub struct EvalEnumModel {
    pub name: Ident,
    pub tag: LitStr,
    pub attr: LitStr,

    pub variants: Vec<EvalVariant>,
    pub none: Option<NoneVariant>,
}

impl EvalEnumModel {
    pub fn new(
        name: Ident,
        tag: LitStr,
        attr: LitStr,
        variants: Vec<EvalVariant>,
        none: Option<NoneVariant>,
    ) -> Self {
        Self {
            name,
            tag,
            attr,
            variants,
            none,
        }
    }

    pub fn expand(self) -> Result<TokenStream> {
        let Self {
            name,
            tag,
            attr,
            variants,
            none,
            ..
        } = self;

        let name_str = name.to_string();

        let mut inner_match_body = TokenStream::new();
        for variant in variants {
            let EvalVariant { name, ty, value } = variant;
            inner_match_body.extend(quote!(
                #value => Self::#name(#ty::from_xml(reader, tag, attrs)?),
            ));
        }

        let none_variant = match none {
            Some(NoneVariant { name, ty }) => {
                quote!( Self::#name(#ty::from_xml(reader, tag, attrs)?),)
            }
            None => {
                quote!({
                    return Err(crate::error::Error::new(
                        xml::common::Position::position(reader),
                        crate::error::ErrorKind::BadEvalVariant {
                            name: #name_str.to_string(),
                            tag,
                            attr: #attr.to_string(),
                            value: None,
                        },
                    ))
                })
            }
        };

        let outer_match_body = quote!(
            Some(value) => match value.as_str() {
                #inner_match_body
                _ => {
                    return Err(crate::error::Error::new(
                        xml::common::Position::position(reader),
                        crate::error::ErrorKind::BadEvalVariant {
                            name: #name_str.to_string(),
                            tag, attr: #attr.to_string(),
                            value: Some(value),
                        },
                    ))
                }
            }
            None => #none_variant
        );

        let fn_body = quote!(
            Ok(match attrs.remove(#attr) {
                #outer_match_body
            })
        );

        Ok(quote!(
            impl #name {
                pub const TAG: &'static str = #tag;
            }

            impl crate::FromXml for #name {
                fn from_xml<R: std::io::Read>(
                    reader: &mut xml::EventReader<R>,
                    tag: String,
                    mut attrs: std::collections::HashMap<String, String>,
                ) -> Result<Self, crate::error::Error> {
                    #fn_body
                }
            }
        ))
    }
}
