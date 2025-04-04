mod enums;

use std::{fs::DirBuilder, path::PathBuf};

use enums::generate_enums_file;
use genvk_core::{Api, Vulkan, error::Error};
use proc_macro2::TokenStream;

pub fn generate_binding(vk: Vulkan) -> Result<(), Error> {
    generate_binding_files(create_root_dir(vk.api)?, vk)
}

fn create_root_dir(api: Api) -> Result<PathBuf, Error> {
    let root_module = match api {
        Api::Vulkan => "vk",
        Api::VulkanSc => "vksc",
    };

    let root = PathBuf::from(format!("output/genvk/auto-generated/{root_module}"));
    if !root.exists() {
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder.create(&root)?;
    }

    Ok(root)
}

fn generate_binding_files(root: PathBuf, vk: Vulkan) -> Result<(), Error> {
    generate_enums_file(&root, &vk)?;
    Ok(())
}

pub(crate) fn write_binding_file(path: PathBuf, contents: TokenStream) -> Result<(), Error> {
    Ok(std::fs::write(
        path,
        prettyplease::unparse(&syn::parse2(contents).map_err(Error::OutSyn)?),
    )?)
}
