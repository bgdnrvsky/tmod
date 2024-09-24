use anyhow::Context;
use dialoguer::{Input, Select};
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

use crate::fetcher::SEARCHER;

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
/// version = "47.2.2" # Either semver version or maven version
/// ```
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Loader {
    kind: Loaders,
    version: String,
}

impl Loader {
    pub fn prompt() -> anyhow::Result<Self> {
        let kind = Loaders::prompt()?;
        let version = Input::<String>::new()
            .with_prompt("Loader version")
            .interact()
            .unwrap()
            .parse()?;

        Self::new_checked(kind, version)
    }

    pub fn new_unchecked(kind: Loaders, version: String) -> Self {
        Self { kind, version }
    }

    pub fn new_checked(kind: Loaders, version: String) -> anyhow::Result<Self> {
        let mut searcher = SEARCHER.try_lock().unwrap();

        let old_silent = searcher.silent();
        searcher.set_silent(true);

        // Check if version exists
        let exists: bool = match kind {
            Loaders::Forge => searcher
                .forge_versions()?
                .values()
                .any(|versions| versions.contains(&version)),
            Loaders::Fabric => searcher.fabric_versions()?.contains(&version),
            // TODO
            Loaders::Quilt => false,
            Loaders::NeoForge => false,
        };

        searcher.set_silent(old_silent);

        anyhow::ensure!(exists, "The version {version} of the {kind} doesn't exist");

        Ok(Self { kind, version })
    }

    pub fn kind(&self) -> Loaders {
        self.kind
    }

    pub fn version(&self) -> &str {
        &self.version
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
            version = "47.2.2"
            "#,
        )
        .map(|_| ())
        .context("Failed to deserialize a valid loader config")
    }

    #[test]
    #[should_panic]
    fn missing_version() {
        toml::from_str::<Loader>(
            r#"
            kind = "fabric"
            "#,
        )
        .expect("Missing version in loader config");
    }

    #[test]
    #[should_panic]
    fn missing_kind() {
        toml::from_str::<Loader>(
            r#"
            version = "47.2.2"
            "#,
        )
        .expect("Missing kind in loader config");
    }
}
