pub mod error;
pub mod parse;

use std::collections::HashMap;

use self::error::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Api {
    Vulkan,
    VulkanSc,
}

impl Api {
    pub fn is_compatible(&self, api: &Option<Vec<parse::util::Api>>) -> bool {
        api.as_ref().is_none_or(|apis| {
            apis.iter().any(|rhs| match (self, rhs) {
                (Api::Vulkan, genvk_parse::util::Api::Vulkan)
                | (Api::VulkanSc, genvk_parse::util::Api::VulkanSc) => true,
                _ => false,
            })
        })
    }
}

pub struct Vulkan {
    pub api: Api,
    pub types: Types,
    pub enums: Enums,
}

impl Vulkan {
    pub fn new(api: Api, registry: &parse::Registry) -> Result<Self, Error> {
        Ok(Self {
            types: Types::new(api, &registry)?,
            enums: Enums::new(api, &registry)?,
            api,
        })
    }
}

pub struct Types {
    pub items: HashMap<String, Type>,
}

impl Types {
    pub fn new(api: Api, registry: &parse::Registry) -> Result<Self, Error> {
        let items = Self::create_types_map(api, registry)?;
        Ok(Self { items })
    }

    fn create_types_map(
        api: Api,
        registry: &parse::Registry,
    ) -> Result<HashMap<String, Type>, Error> {
        let mut out = HashMap::new();
        for types in registry.items.iter().filter_map(|item| match item {
            parse::RegistryItem::Types(types) => Some(types),
            _ => None,
        }) {
            Self::process_registry_types(api, types, &mut out)?;
        }

        Ok(out)
    }

    fn process_registry_types(
        api: Api,
        types: &parse::Types,
        out: &mut HashMap<String, Type>,
    ) -> Result<(), Error> {
        for item in types.items.iter().filter_map(|item| match item {
            parse::TypesItem::Type(item) => Some(item),
            parse::TypesItem::Comment(_) => None,
        }) {
            Self::process_registry_type(api, item, out)?;
        }

        Ok(())
    }

    fn process_registry_type(
        api: Api,
        ty: &parse::Type,
        out: &mut HashMap<String, Type>,
    ) -> Result<(), Error> {
        Ok(match ty {
            parse::Type::Alias(_) => {}
            parse::Type::Explicit(ty) => match ty {
                parse::ExplicitType::Enum(ty) => {
                    if api.is_compatible(&ty.api) {
                        let output = OutputTypeEnum {
                            name: ty.name.clone(),
                        };
                        out.insert(
                            ty.name.clone(),
                            Type {
                                name: ty.name.clone(),
                                output: OutputType::Enum(output),
                            },
                        );
                    }
                }
                _ => {}
            },
        })
    }
}

pub struct Type {
    pub name: String,
    pub output: OutputType,
}

pub enum OutputType {
    Enum(OutputTypeEnum),
}

pub struct OutputTypeEnum {
    pub name: String,
}

pub struct Enums {
    pub items: HashMap<String, EnumBlocks>,
}

impl Enums {
    pub fn new(_api: Api, _registry: &parse::Registry) -> Result<Self, Error> {
        Ok(Self {
            items: HashMap::new(),
        })
    }
}

pub struct EnumBlocks {}
