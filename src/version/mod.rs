pub mod maven;

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

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum ManyVersions {
    Fabric(FabricVersionRange),
    Forge(ForgeVersionRange),
}
