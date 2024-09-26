use anyhow::Context;
use dialoguer::Select;
use serde::{Deserialize, Serialize};
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
    fn prompt() -> anyhow::Result<Self> {
        let loaders = Self::iter().collect::<Vec<_>>();

        Ok(loaders[Select::new()
            .with_prompt("Choose the mod loader")
            .items(&loaders)
            .interact()
            .context("Error when prompting loader")?])
    }
}

/// Configuration unit for describing the mod management system used and its version
///
/// Example config:
/// ```toml
/// kind = "forge" # any case accepted (e.g. FORGE, FoRgE)
/// ```
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Loader {
    kind: Loaders,
}

impl Loader {
    pub fn prompt() -> anyhow::Result<Self> {
        Loaders::prompt().map(|kind| Self { kind })
    }

    pub fn kind(&self) -> Loaders {
        self.kind
    }
}

#[cfg(test)]
mod loader_deserializing {
    use anyhow::Context;

    use super::Loader;

    #[test]
    fn valid() -> anyhow::Result<()> {
        toml::from_str::<Loader>(
            r#"
            kind = "forge"
            "#,
        )
        .map(|_| ())
        .context("Failed to deserialize a valid loader config")
    }
}
