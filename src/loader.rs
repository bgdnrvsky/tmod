use serde_with::DeserializeFromStr;
use strum::{Display, EnumString};

#[derive(Debug, Clone, EnumString, DeserializeFromStr, Display, PartialEq, Eq)]
#[strum(ascii_case_insensitive)]
pub enum Loader {
    Forge,
    Fabric,
    Quilt,
    NeoForge,
}

#[cfg(test)]
mod loaders_kind_parsing {
    use crate::loader::Loader;
    use std::str::FromStr;

    #[test]
    fn from_lowercase() {
        assert_eq!(Ok(Loader::Forge), Loader::from_str("forge"));
        assert_eq!(Ok(Loader::Fabric), Loader::from_str("fabric"));
        assert_eq!(Ok(Loader::Quilt), Loader::from_str("quilt"));
        assert_eq!(Ok(Loader::NeoForge), Loader::from_str("neoforge"));
    }

    #[test]
    fn from_uppercase() {
        assert_eq!(Ok(Loader::Forge), Loader::from_str("FORGE"));
        assert_eq!(Ok(Loader::Fabric), Loader::from_str("FABRIC"));
        assert_eq!(Ok(Loader::Quilt), Loader::from_str("QUILT"));
        assert_eq!(Ok(Loader::NeoForge), Loader::from_str("NEOFORGE"));
    }

    #[test]
    fn from_mixedcase() {
        assert_eq!(Ok(Loader::Forge), Loader::from_str("fOrGe"));
        assert_eq!(Ok(Loader::Fabric), Loader::from_str("FabRIC"));
        assert_eq!(Ok(Loader::Quilt), Loader::from_str("QUIlT"));
        assert_eq!(Ok(Loader::NeoForge), Loader::from_str("NeOFORGE"));
    }

    #[test]
    fn from_invalid() {
        assert!(Loader::from_str("loader").is_err());
        assert!(Loader::from_str("LOADER").is_err());
        assert!(Loader::from_str("LoAder").is_err());
    }
}
