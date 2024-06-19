pub mod maven;

use std::fmt::Display;

use maven::Version as ForgeVersion;
use semver::Version as FabricVersion;

use maven::VersionRange as ForgeVersionRange;
use semver::VersionReq as FabricVersionRange;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum ManyVersions {
    Fabric(FabricVersionRange),
    Forge(ForgeVersionRange),
}

impl Display for ManyVersions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManyVersions::Fabric(version) => write!(f, "{version}"),
            ManyVersions::Forge(version) => write!(f, "{version}"),
        }
    }
}
