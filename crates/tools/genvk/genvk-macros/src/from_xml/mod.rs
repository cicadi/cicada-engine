pub mod attr;
pub mod expand;
pub mod util;

use self::{
    attr::{EnumAttr, FieldAttr, StructAttr, VariantAttr},
    expand::{
        EvalEnumModel, EvalVariant, Field, Items, NoneVariant, StructModel, TaggedEnumModel,
        TaggedVariant, Text, VarEnumModel,
    },
    util::parse_attributes,
};

use expand::VarVariant;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Error, Ident, LitStr, Result, Type,
    spanned::Spanned,
};

pub fn expand(input: DeriveInput) -> Result<TokenStream> {
    match input.data {
        Data::Struct(data) => expand_struct(input.ident, input.attrs, data),
        Data::Enum(data) => expand_enum(input.ident, input.attrs, data),
        Data::Union(data) => Err(Error::new(
            data.union_token.span,
            "`#[derive(FromXml)]` not supported for `union` types",
        )),
    }
}

fn expand_struct(name: Ident, attrs: Vec<Attribute>, data: DataStruct) -> Result<TokenStream> {
    let mut tag = None;
    for a in parse_attributes(&attrs)? {
        match a {
            StructAttr::Tag(value) => tag = Some(value),
        }
    }

    let Some(tag) = tag else {
        return Err(Error::new(
            Span::call_site(),
            "Tag not specified for deriving struct\n\
            Structs deriving `FromXml` must have attribute #[vk(`tag = \"tag name\"`)]",
        ));
    };

    enum FieldKind {
        Attribute(Option<LitStr>),
        Items(Type),
        Text,
        Generic,
    }

    let mut fields = Vec::new();
    let mut items = None;
    let mut text = None;
    let mut generic = None;
    for field in data.fields {
        let mut kind = None;
        let mut optional = None;
        for a in parse_attributes(&field.attrs)? {
            match a {
                FieldAttr::Name(ident, value) => {
                    kind = match kind {
                        None | Some(FieldKind::Attribute(None)) => {
                            Some(FieldKind::Attribute(Some(value)))
                        }
                        Some(FieldKind::Attribute(Some(..))) => {
                            return Err(Error::new(
                                ident.span(),
                                format!("attribute `{ident}` is repeated"),
                            ));
                        }
                        _ => {
                            return Err(Error::new(
                                ident.span(),
                                format!("attribute `{ident}` not allowed here",),
                            ));
                        }
                    }
                }
                FieldAttr::Optional(ident) => {
                    if let None = optional {
                        optional = Some(true);
                        kind = match kind {
                            None => Some(FieldKind::Attribute(None)),
                            kind @ Some(FieldKind::Attribute(..)) => kind,
                            _ => {
                                return Err(Error::new(
                                    ident.span(),
                                    format!("attribute `{ident}` not allowed here",),
                                ));
                            }
                        }
                    } else {
                        return Err(Error::new(
                            ident.span(),
                            format!("attribute `{ident}` is repeated"),
                        ));
                    }
                }
                FieldAttr::Items(ident, value) => match kind {
                    None => kind = Some(FieldKind::Items(value)),
                    Some(FieldKind::Items(..)) => {
                        return Err(Error::new(
                            ident.span(),
                            format!("attribute `{ident}` is repeated"),
                        ));
                    }
                    _ => {
                        return Err(Error::new(
                            ident.span(),
                            format!("attribute `{ident}` not allowed here",),
                        ));
                    }
                },
                FieldAttr::Text(ident) => match kind {
                    None => kind = Some(FieldKind::Text),
                    Some(FieldKind::Text) => {
                        return Err(Error::new(
                            ident.span(),
                            format!("attribute `{ident}` is repeated"),
                        ));
                    }
                    _ => {
                        return Err(Error::new(
                            ident.span(),
                            format!("attribute `{ident}` not allowed here",),
                        ));
                    }
                },
                FieldAttr::Generic(ident) => match kind {
                    None => kind = Some(FieldKind::Generic),
                    Some(FieldKind::Generic) => {
                        return Err(Error::new(
                            ident.span(),
                            format!("attribute `{ident}` is repeated"),
                        ));
                    }
                    _ => {
                        return Err(Error::new(
                            ident.span(),
                            format!("attribute `{ident}` not allowed here",),
                        ));
                    }
                },
            }
        }

        let kind = match kind {
            Some(kind) => kind,
            None => FieldKind::Attribute(None),
        };

        match kind {
            FieldKind::Attribute(attr) => {
                let field_name = field.ident.clone().ok_or(Error::new(
                    field.span(),
                    "attribute field not allowed in struct types",
                ))?;
                let attr = attr.unwrap_or_else(|| {
                    LitStr::new(field_name.to_string().as_str(), Span::call_site())
                });

                fields.push(Field {
                    name: field_name,
                    attr,
                    ty: field.ty,
                    optional: optional.unwrap_or(false),
                });
            }
            FieldKind::Items(ty) => {
                let field_name = field.ident.clone().ok_or(Error::new(
                    field.span(),
                    "attribute field not allowed in struct types",
                ))?;

                items = Some(Items {
                    name: field_name,
                    ty,
                })
            }
            FieldKind::Text => {
                let field_name = field.ident.clone().ok_or(Error::new(
                    field.span(),
                    "attribute field not allowed in struct types",
                ))?;

                text = Some(Text {
                    name: field_name,
                    generic: false,
                });
            }
            FieldKind::Generic => {
                let field_name = field.ident.clone().ok_or(Error::new(
                    field.span(),
                    "attribute field not allowed in struct types",
                ))?;

                generic = Some(field_name);
            }
        }
    }

    if generic.is_some() && (items.is_some() || text.is_some()) {
        Err(Error::new(
            Span::call_site(),
            "invalid configuration of fields",
        ))
    } else {
        if let Some(field) = generic {
            if text.is_some() {
                return Err(Error::new(
                    Span::call_site(),
                    "invalid configuration of fields -- simultaneous `generic` and `text` fields not allowed",
                ));
            } else {
                text = Some(Text {
                    name: field.clone(),
                    generic: true,
                })
            }

            if items.is_some() {
                return Err(Error::new(
                    Span::call_site(),
                    "invalid configuration of fields -- simultaneous `generic` and `items` fields not allowed",
                ));
            } else {
                items = Some(Items {
                    name: field.clone(),
                    ty: syn::parse(quote!(GenericItem).into())?,
                })
            }
        }

        StructModel::new(name, tag, fields, items, text).expand()
    }
}

fn expand_enum(name: Ident, attrs: Vec<Attribute>, data: DataEnum) -> Result<TokenStream> {
    enum EnumKind {
        Tagged,
        Var,
        Eval,
    }

    let mut kind = None;
    let mut tag = None;
    let mut attr = None;
    for a in parse_attributes(&attrs)? {
        match a {
            EnumAttr::Tagged(ident) => {
                kind = match kind {
                    None => Some(EnumKind::Tagged),
                    _ => {
                        return Err(Error::new(
                            ident.span(),
                            format!("attribute `{ident}` not allowed here",),
                        ));
                    }
                }
            }
            EnumAttr::Var(ident) => {
                kind = match kind {
                    None => Some(EnumKind::Var),
                    _ => {
                        return Err(Error::new(
                            ident.span(),
                            format!("attribute `{ident}` not allowed here",),
                        ));
                    }
                }
            }
            EnumAttr::Eval(ident) => {
                kind = match kind {
                    None => Some(EnumKind::Eval),
                    _ => {
                        return Err(Error::new(
                            ident.span(),
                            format!("attribute `{ident}` not allowed here",),
                        ));
                    }
                }
            }

            EnumAttr::Tag(ident, value) => match kind {
                Some(EnumKind::Var | EnumKind::Eval) => tag = Some(value),
                _ => {
                    return Err(Error::new(
                        ident.span(),
                        format!("attribute `{ident}` not allowed here",),
                    ));
                }
            },
            EnumAttr::Attr(ident, value) => match kind {
                Some(EnumKind::Eval) => attr = Some(value),
                _ => {
                    return Err(Error::new(
                        ident.span(),
                        format!("attribute `{ident}` not allowed here",),
                    ));
                }
            },
        }
    }

    if let Some(kind) = kind {
        match kind {
            EnumKind::Tagged => expand_tagged_enum(name, data),
            EnumKind::Var => {
                let Some(tag) = tag else {
                    return Err(Error::new(Span::call_site(), "Tag not specified"));
                };
                expand_var_enum(name, tag, data)
            }
            EnumKind::Eval => {
                let Some(tag) = tag else {
                    return Err(Error::new(Span::call_site(), "Tag not specified"));
                };
                let Some(attr) = attr else {
                    return Err(Error::new(
                        Span::call_site(),
                        "Attribute name not specified",
                    ));
                };

                expand_eval_enum(name, tag, attr, data)
            }
        }
    } else {
        Err(Error::new(
            Span::call_site(),
            "Enum kind not specified -- must specify `tagged`, `var` or `eval`",
        ))
    }
}

fn expand_tagged_enum(name: Ident, data: DataEnum) -> Result<TokenStream> {
    let mut variants = Vec::new();
    for variant in data.variants {
        let fields = variant.fields.iter().collect::<Vec<_>>();
        let ty = match fields.as_slice() {
            [field] => field.ty.clone(),
            _ => {
                return Err(Error::new(
                    variant.span(),
                    "Variants without exactly a single type are not allowed",
                ));
            }
        };

        variants.push(TaggedVariant {
            name: variant.ident,
            ty,
        });
    }

    TaggedEnumModel::new(name, variants).expand()
}

fn expand_var_enum(name: Ident, tag: LitStr, data: DataEnum) -> Result<TokenStream> {
    enum VariantKind {
        Attr(LitStr),
        None,
    }

    let mut variants = Vec::new();
    let mut none = None;
    for variant in data.variants {
        let fields = variant.fields.iter().collect::<Vec<_>>();
        let ty = match fields.as_slice() {
            [field] => field.ty.clone(),
            _ => {
                return Err(Error::new(
                    variant.span(),
                    "Variants without exactly a single type are not allowed",
                ));
            }
        };

        let mut kind = None;
        for a in parse_attributes(&variant.attrs)? {
            match a {
                VariantAttr::Value(ident, ..) => {
                    return Err(Error::new(
                        ident.span(),
                        format!("attribute `{ident}` not allowed here",),
                    ));
                }
                VariantAttr::Attr(ident, value) => {
                    kind = match kind {
                        None => Some(VariantKind::Attr(value)),
                        Some(VariantKind::Attr(..)) => {
                            return Err(Error::new(
                                ident.span(),
                                format!("attribute `{ident}` is repeated"),
                            ));
                        }
                        _ => {
                            return Err(Error::new(
                                ident.span(),
                                format!("attribute `{ident}` not allowed here",),
                            ));
                        }
                    };
                }
                VariantAttr::None(ident) => {
                    kind = match kind {
                        None => Some(VariantKind::None),
                        Some(VariantKind::None) => {
                            return Err(Error::new(
                                ident.span(),
                                format!("attribute `{ident}` is repeated"),
                            ));
                        }
                        _ => {
                            return Err(Error::new(
                                ident.span(),
                                format!("attribute `{ident}` not allowed here",),
                            ));
                        }
                    };
                }
            }
        }

        match kind {
            Some(kind) => match kind {
                VariantKind::Attr(attr) => variants.push(VarVariant {
                    name: variant.ident,
                    ty,
                    attr,
                }),
                VariantKind::None => {
                    none = match none {
                        None => Some(NoneVariant {
                            name: variant.ident,
                            ty,
                        }),
                        _ => {
                            return Err(Error::new(
                                variant.span(),
                                format!("repeated `none` variants"),
                            ));
                        }
                    }
                }
            },
            None => {
                return Err(Error::new(
                    variant.span(),
                    format!("purpose of variant unspecified"),
                ));
            }
        }
    }

    VarEnumModel::new(name, tag, variants, none).expand()
}

fn expand_eval_enum(name: Ident, tag: LitStr, attr: LitStr, data: DataEnum) -> Result<TokenStream> {
    enum VariantKind {
        Value(LitStr),
        None,
    }

    let mut variants = Vec::new();
    let mut none = None;
    for variant in data.variants {
        let fields = variant.fields.iter().collect::<Vec<_>>();
        let ty = match fields.as_slice() {
            [field] => field.ty.clone(),
            _ => {
                return Err(Error::new(
                    variant.span(),
                    "Variants without exactly a single type are not allowed",
                ));
            }
        };

        let mut kind = None;
        for a in parse_attributes(&variant.attrs)? {
            match a {
                VariantAttr::Attr(ident, ..) => {
                    return Err(Error::new(
                        ident.span(),
                        format!("attribute `{ident}` not allowed here",),
                    ));
                }
                VariantAttr::Value(ident, value) => {
                    kind = match kind {
                        None => Some(VariantKind::Value(value)),
                        Some(VariantKind::Value(..)) => {
                            return Err(Error::new(
                                ident.span(),
                                format!("attribute `{ident}` is repeated"),
                            ));
                        }
                        _ => {
                            return Err(Error::new(
                                ident.span(),
                                format!("attribute `{ident}` not allowed here",),
                            ));
                        }
                    };
                }
                VariantAttr::None(ident) => {
                    kind = match kind {
                        None => Some(VariantKind::None),
                        Some(VariantKind::None) => {
                            return Err(Error::new(
                                ident.span(),
                                format!("attribute `{ident}` is repeated"),
                            ));
                        }
                        _ => {
                            return Err(Error::new(
                                ident.span(),
                                format!("attribute `{ident}` not allowed here",),
                            ));
                        }
                    };
                }
            }
        }

        match kind {
            Some(kind) => match kind {
                VariantKind::Value(value) => variants.push(EvalVariant {
                    name: variant.ident,
                    ty,
                    value,
                }),
                VariantKind::None => {
                    none = match none {
                        None => Some(NoneVariant {
                            name: variant.ident,
                            ty,
                        }),
                        _ => {
                            return Err(Error::new(
                                variant.span(),
                                format!("repeated `none` variants"),
                            ));
                        }
                    }
                }
            },
            None => {
                return Err(Error::new(
                    variant.span(),
                    format!("purpose of variant unspecified"),
                ));
            }
        }
    }

    EvalEnumModel::new(name, tag, attr, variants, none).expand()
}
