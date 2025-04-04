pub mod enums;
pub mod error;
pub mod parse;
pub mod types;

use self::{error::Error, types::Types};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Api {
    Vulkan,
    VulkanSc,
}

impl Api {
    pub fn is_compatible_with<T: AsRef<[parse::Api]>>(&self, api: &Option<T>) -> bool {
        api.as_ref().is_none_or(|apis| {
            apis.as_ref().iter().any(|rhs| match (self, rhs) {
                (Api::Vulkan, parse::Api::Vulkan) | (Api::VulkanSc, parse::Api::VulkanSc) => true,
                _ => false,
            })
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum Deprecation {
    #[default]
    False,
    True,
    Aliased,
    Ignored,
}

impl From<Option<parse::Deprecation>> for Deprecation {
    fn from(value: Option<parse::Deprecation>) -> Self {
        match value {
            None => Self::False,
            Some(parse::Deprecation::True) => Self::True,
            Some(parse::Deprecation::Ignored) => Self::Ignored,
            Some(parse::Deprecation::Aliased) => Self::Aliased,
        }
    }
}

pub struct Vulkan {
    pub api: Api,
    pub types: Types,
}

impl Vulkan {
    pub fn new(api: Api) -> Self {
        Self {
            api,
            types: Default::default(),
        }
    }

    pub fn process(mut self, registry: &parse::Registry) -> Result<Self, Error> {
        let mut ctxt = Ctxt::new(self.api);
        loop {
            self.populate_with_ctxt(&mut ctxt, registry)?;
            if !ctxt.loop_needed {
                break;
            } else if !ctxt.is_updated {
                return Err(Error::PopulateLoop);
            } else {
                ctxt.reset();
            }
        }

        Ok(self)
    }

    fn populate_with_ctxt(
        &mut self,
        ctxt: &mut Ctxt,
        registry: &parse::Registry,
    ) -> Result<(), Error> {
        use parse::RegistryItem::*;

        println!("started population loop #{}", ctxt.loops_count);
        for item in registry.items.iter() {
            match item {
                Comment(_) => {}

                Types(types) => self.types.process_types(ctxt, types)?,
                _ => {}
            }
        }

        Ok(())
    }
}

pub(crate) struct Ctxt {
    pub api: Api,

    pub loop_needed: bool,
    pub is_updated: bool,

    pub loops_count: u32,
}

impl Ctxt {
    pub(crate) fn new(api: Api) -> Self {
        Self {
            api,

            loop_needed: false,
            is_updated: false,

            loops_count: 0,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.loop_needed = false;
        self.is_updated = false;

        self.loops_count += 1;
    }
}
