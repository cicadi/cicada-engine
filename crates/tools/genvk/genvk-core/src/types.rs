use std::{collections::HashMap, num::NonZeroU32};

use crate::{Ctxt, Deprecation, enums::EnumId, error::Error, parse};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub(crate) NonZeroU32);

impl TypeId {
    pub fn new(id: NonZeroU32) -> Self {
        Self(id)
    }

    pub fn from_u32(value: u32) -> Option<Self> {
        NonZeroU32::new(value).map(Self)
    }

    pub fn into_inner(self) -> NonZeroU32 {
        self.0
    }

    pub fn into_u32(self) -> u32 {
        self.into_inner().get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpecialType {
    Int,

    VkResult,
}

impl SpecialType {
    pub fn from_name(name: &str) -> Option<SpecialType> {
        use SpecialType::*;
        Some(match name {
            "int" => Int,

            "VkResult" => VkResult,

            _ => return None,
        })
    }
}

pub struct Types {
    pub name_to_id: HashMap<String, TypeId>,
    pub id_to_type: HashMap<TypeId, Type>,

    pub handle_type_ids: Vec<TypeId>,
    pub enum_type_ids: Vec<TypeId>,
    pub bitmask_type_ids: Vec<TypeId>,
    pub struct_type_ids: Vec<TypeId>,
    pub union_type_ids: Vec<TypeId>,
    pub fn_ptr_type_ids: Vec<TypeId>,

    pub special_type_ids: HashMap<SpecialType, TypeId>,

    pub next_free_id: NonZeroU32,
}

impl Default for Types {
    fn default() -> Self {
        Self {
            name_to_id: HashMap::new(),
            id_to_type: HashMap::new(),

            handle_type_ids: Vec::new(),
            enum_type_ids: Vec::new(),
            bitmask_type_ids: Vec::new(),
            struct_type_ids: Vec::new(),
            union_type_ids: Vec::new(),
            fn_ptr_type_ids: Vec::new(),

            special_type_ids: HashMap::new(),

            next_free_id: NonZeroU32::MIN,
        }
    }
}

impl Types {
    fn register(&mut self, ty: Type) -> Result<TypeId, Error> {
        if self.name_to_id.contains_key(&ty.api_name) {
            Err(Error::DuplicateType)
        } else {
            let prev_id = self.next_free_id;
            let id = self.next_id();

            if let Some(special) = SpecialType::from_name(&ty.api_name) {
                if !self.special_type_ids.contains_key(&special) {
                    self.special_type_ids.insert(special, id);
                } else {
                    self.next_free_id = prev_id; // reset back to not waste free ids
                    return Err(Error::invalid_state(
                        "[impossible branch] special type already registered",
                    ));
                }
            }

            match &ty.kind {
                TypeKind::Handle => self.handle_type_ids.push(id),
                TypeKind::Enum(_) => self.enum_type_ids.push(id),
                TypeKind::Bitmask => self.bitmask_type_ids.push(id),
                TypeKind::Struct => self.struct_type_ids.push(id),
                TypeKind::Union => self.union_type_ids.push(id),
                TypeKind::FnPtr => self.fn_ptr_type_ids.push(id),
                _ => {}
            }

            self.name_to_id.insert(ty.api_name.clone(), id);
            self.id_to_type.insert(id, ty);

            Ok(id)
        }
    }

    fn next_id(&mut self) -> TypeId {
        let out = TypeId::new(self.next_free_id);
        self.next_free_id = self.next_free_id.saturating_add(1);
        out
    }

    pub fn contains(&self, name: &str) -> bool {
        self.name_to_id.contains_key(name)
    }

    pub fn rustify(&self, api_name: &str) -> String {
        if api_name.starts_with("Vk") {
            api_name[2..].to_string()
        } else {
            api_name.to_string()
        }
    }

    pub(crate) fn process_types(
        &mut self,
        ctxt: &mut Ctxt,
        types: &parse::Types,
    ) -> Result<(), Error> {
        for item in types.items.iter().filter_map(|item| match item {
            parse::TypesItem::Type(item) => Some(item),
            parse::TypesItem::Comment(_) => None,
        }) {
            Type::process(self, ctxt, item)?;
        }

        Ok(())
    }
}

pub struct Type {
    pub api_name: String,
    pub kind: TypeKind,

    pub deprecated: Deprecation,
}

impl Type {
    pub fn new(api_name: String, kind: TypeKind, deprecated: Deprecation) -> Self {
        Self {
            api_name,
            kind,
            deprecated,
        }
    }

    fn process(types: &mut Types, ctxt: &mut Ctxt, item: &parse::Type) -> Result<(), Error> {
        use parse::{ExplicitType::*, Type::*};
        Ok(match item {
            Alias(_) => {} // TODO: alias types
            Explicit(item) => {
                if ctxt.api.is_compatible_with(&item.api()) {
                    if let Some(requires) = item.requires() {
                        if !types.name_to_id.contains_key(requires) {
                            println!("skipping a type because type `{requires}` is missing.");
                            ctxt.loop_needed = true;
                            return Ok(());
                        }
                    }

                    match item {
                        Enum(item) => TypeEnum::process(types, ctxt, item)?,
                        _ => {}
                    }
                }
            }
        })
    }
}

pub enum TypeKind {
    Include,
    Define,

    Fund,
    System,

    Base,
    Handle,

    Enum(TypeEnum),
    Bitmask,

    Struct,
    Union,
    FnPtr,
}

pub struct TypeEnum {
    pub output_name: String,
    pub enum_id: Option<EnumId>,
}

impl TypeEnum {
    pub fn new(output_name: String) -> Self {
        Self {
            output_name,
            enum_id: None,
        }
    }

    fn process(types: &mut Types, ctxt: &mut Ctxt, item: &parse::TypeEnum) -> Result<(), Error> {
        if !types.contains(&item.name) {
            let ty = Self::new(types.rustify(&item.name));
            types.register(Type::new(
                item.name.clone(),
                TypeKind::Enum(ty),
                item.deprecated.into(),
            ))?;

            ctxt.is_updated = true;
        }

        Ok(())
    }
}
