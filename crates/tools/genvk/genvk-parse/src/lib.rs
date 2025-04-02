pub mod core;
pub mod error;
pub mod macros;
pub mod registry;
pub mod util;

use std::{collections::HashMap, io::Read};

use xml::EventReader;

use crate::error::Error;

pub trait FromXml: Sized {
    fn from_xml<R: Read>(
        reader: &mut EventReader<R>,
        tag: String,
        attrs: HashMap<String, String>,
    ) -> Result<Self, Error>;
}
