use crate::version::SingleVersion;

use anyhow::anyhow;
use serde::Deserialize;
use serde_with::DeserializeFromStr;
use strum::{Display, EnumString};

/// Various mod management systems for Minecraft
#[derive(Debug, Clone, EnumString, DeserializeFromStr, Display, PartialEq, Eq)]
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
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Loader {
    kind: Loaders,
    version: SingleVersion,
}

impl Loader {
    pub fn new(kind: Loaders, version: SingleVersion) -> anyhow::Result<Self> {
        // TODO: Check if version exists
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

#[cfg(test)]
mod loaders_kind_parsing {
    use crate::loader::Loaders;
    use std::str::FromStr;

    #[test]
    fn from_lowercase() {
        assert_eq!(Ok(Loaders::Forge), Loaders::from_str("forge"));
        assert_eq!(Ok(Loaders::Fabric), Loaders::from_str("fabric"));
        assert_eq!(Ok(Loaders::Quilt), Loaders::from_str("quilt"));
        assert_eq!(Ok(Loaders::NeoForge), Loaders::from_str("neoforge"));
    }

    #[test]
    fn from_uppercase() {
        assert_eq!(Ok(Loaders::Forge), Loaders::from_str("FORGE"));
        assert_eq!(Ok(Loaders::Fabric), Loaders::from_str("FABRIC"));
        assert_eq!(Ok(Loaders::Quilt), Loaders::from_str("QUILT"));
        assert_eq!(Ok(Loaders::NeoForge), Loaders::from_str("NEOFORGE"));
    }

    #[test]
    fn from_mixedcase() {
        assert_eq!(Ok(Loaders::Forge), Loaders::from_str("fOrGe"));
        assert_eq!(Ok(Loaders::Fabric), Loaders::from_str("FabRIC"));
        assert_eq!(Ok(Loaders::Quilt), Loaders::from_str("QUIlT"));
        assert_eq!(Ok(Loaders::NeoForge), Loaders::from_str("NeOFORGE"));
    }

    #[test]
    fn from_invalid() {
        assert!(Loaders::from_str("loader").is_err());
        assert!(Loaders::from_str("LOADER").is_err());
        assert!(Loaders::from_str("LoAder").is_err());
    }
}
