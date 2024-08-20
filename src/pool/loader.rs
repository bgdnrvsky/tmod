use crate::version::SingleVersion;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum::{Display, EnumString};

/// Various mod management systems for Minecraft
#[derive(
    Debug, Clone, EnumString, DeserializeFromStr, SerializeDisplay, Display, PartialEq, Eq,
)]
#[strum(ascii_case_insensitive)]
pub enum Loaders {
    Forge,
    Fabric,
    Quilt,
    NeoForge,
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
    version: SingleVersion,
}

impl Loader {
    pub fn new_unchecked(kind: Loaders, version: SingleVersion) -> Self {
        Self { kind, version }
    }

    pub fn new_checked(
        kind: Loaders,
        version: SingleVersion,
        searcher: crate::fetcher::searcher::Searcher,
    ) -> anyhow::Result<Self> {
        // Check if version exists
        let exists: bool = match version {
            SingleVersion::Forge(_) => searcher
                .forge_versions()?
                .values()
                .any(|versions| versions.contains(&version)),
            SingleVersion::Fabric(_) => searcher.fabric_versions()?.contains(&version),
        };

        anyhow::ensure!(exists, "The version {version} of the {kind} doesn't exist");

        Ok(Self { kind, version })
    }
}

impl TryFrom<usize> for Loaders {
    type Error = anyhow::Error;

    fn try_from(loader_id: usize) -> Result<Self, Self::Error> {
        match loader_id {
            1 => Ok(Self::Forge),
            4 => Ok(Self::Fabric),
            5 => Ok(Self::Quilt),
            6 => Ok(Self::NeoForge),
            _ => Err(anyhow!(
                "Unknown mod loader number {loader_id}\
                             while processing JSON response\
                             while searching for a mod"
            )),
        }
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
