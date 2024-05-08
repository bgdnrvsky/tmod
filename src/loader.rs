use semver::Version;
use serde::Deserialize;
use strum::{Display, EnumString};

#[derive(Debug, Clone, EnumString, Deserialize, Display, PartialEq, Eq)]
#[strum(ascii_case_insensitive)]
pub enum Loaders {
    Forge,
    Fabric,
    Quilt,
    NeoForge,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Loader {
    kind: Loaders,
    version: Version,
}

#[cfg(test)]
mod loaders_kind_parsing {
    use std::str::FromStr;

    use crate::loader::Loaders;

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
