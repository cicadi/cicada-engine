use std::num::NonZeroU32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnumId(pub(crate) NonZeroU32);

impl EnumId {
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
