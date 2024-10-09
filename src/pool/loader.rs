use anyhow::Context;
use dialoguer::Select;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

/// Various mod management systems for Minecraft
#[derive(
    Debug,
    Copy,
    Clone,
    EnumString,
    DeserializeFromStr,
    SerializeDisplay,
    Display,
    PartialEq,
    Eq,
    EnumIter,
)]
#[strum(ascii_case_insensitive)]
#[repr(u8)]
pub enum Loaders {
    Forge = 1,
    Fabric = 4,
    Quilt = 5,
    NeoForge = 6,
}

impl Loaders {
    pub fn prompt() -> anyhow::Result<Self> {
        let loaders = Self::iter().collect::<Vec<_>>();

        Ok(loaders[Select::new()
            .with_prompt("Choose the mod loader")
            .items(&loaders)
            .interact()
            .context("Error when prompting loader")?])
    }
}
