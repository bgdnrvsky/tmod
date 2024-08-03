pub mod fabric;
pub mod maven;

use std::fmt::Display;

use fabric::Version as FabricVersion;
use maven::Version as ForgeVersion;

use fabric::VersionReq as FabricVersionRange;
use maven::VersionRange as ForgeVersionRange;
use serde::{Deserialize, Serialize};

/// Describes a specific version following either maven's or semver's syntax
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(untagged)]
pub enum SingleVersion {
    Fabric(FabricVersion),
    Forge(ForgeVersion),
}

impl Display for SingleVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SingleVersion::Fabric(version) => write!(f, "{version}"),
            SingleVersion::Forge(version) => write!(f, "{version}"),
        }
    }
}

impl PartialEq<SingleVersion> for &SingleVersion {
    fn eq(&self, other: &SingleVersion) -> bool {
        match (self, other) {
            (SingleVersion::Fabric(a), SingleVersion::Fabric(b)) => a.eq(b),
            (SingleVersion::Forge(a), SingleVersion::Forge(b)) => a.eq(b),
            (SingleVersion::Fabric(_), SingleVersion::Forge(_))
            | (SingleVersion::Forge(_), SingleVersion::Fabric(_)) => {
                unimplemented!("Comparing versions of two different breeds is not yet implemented")
            }
        }
    }
}

/// Describes a version range following either maven's or semver's syntax
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum MultiVersion {
    /// See: `https://semver.org`
    Fabric(FabricVersionRange),
    /// See: `https://maven.apache.org/enforcer/enforcer-rules/versionRanges.html`
    Forge(ForgeVersionRange),
}

impl PartialEq<MultiVersion> for &MultiVersion {
    fn eq(&self, other: &MultiVersion) -> bool {
        match (self, other) {
            (MultiVersion::Fabric(a), MultiVersion::Fabric(b)) => a.eq(b),
            (MultiVersion::Forge(a), MultiVersion::Forge(b)) => a.eq(b),
            (MultiVersion::Fabric(_), MultiVersion::Forge(_))
            | (MultiVersion::Forge(_), MultiVersion::Fabric(_)) => {
                unimplemented!(
                    "Comparing version ranges of two different breeds is not yet implemented"
                )
            }
        }
    }
}

impl Display for MultiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MultiVersion::Fabric(version) => write!(f, "{version}"),
            MultiVersion::Forge(version) => write!(f, "{version}"),
        }
    }
}
