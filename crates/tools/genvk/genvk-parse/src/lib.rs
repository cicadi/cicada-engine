pub mod error;
pub mod macros;
pub mod registry;
pub mod util;

use std::{collections::HashMap, io::Read};

use xml::{EventReader, common::Position, reader::ParserConfig2};

use crate::{
    error::{Error, ErrorKind},
    registry::{Registry, VecAttributesExt},
};

pub trait FromXml: Sized {
    fn from_xml<R: Read>(
        reader: &mut EventReader<R>,
        tag: String,
        attrs: HashMap<String, String>,
    ) -> Result<Self, Error>;
}

pub fn parse_xml_source<R: Read>(source: R) -> Result<Registry, Error> {
    let mut reader = xml::EventReader::new_with_config(
        source,
        ParserConfig2::new()
            .coalesce_characters(true)
            .whitespace_to_characters(true)
            .cdata_to_characters(true)
            .ignore_root_level_whitespace(true),
    );

    let mut registry = None;
    loop {
        match reader.next()? {
            xml::reader::XmlEvent::EndDocument => break,
            xml::reader::XmlEvent::StartElement {
                name, attributes, ..
            } => {
                registry = Some(Registry::from_xml(
                    &mut reader,
                    name.local_name,
                    attributes.into_hash_map(),
                )?)
            }
            _ => {}
        }
    }

    registry.ok_or(Error::new(reader.position(), ErrorKind::UnexpectedDocEnd))
}
