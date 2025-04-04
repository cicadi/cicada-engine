use std::fs::File;

use genvk::{
    core::{Api, Vulkan, error::Error},
    output::generate_binding,
    parse::parse_xml_source,
};

fn main() -> Result<(), Error> {
    let registry = parse_xml_source(File::open("vk.xml")?)?;
    for api in [Api::Vulkan, Api::VulkanSc] {
        generate_binding(Vulkan::new(api).process(&registry)?)?;
    }

    Ok(())
}
