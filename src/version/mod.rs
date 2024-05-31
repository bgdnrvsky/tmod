pub mod maven;

use maven::Version as ForgeVersion;
use semver::Version as FabricVersion;

use maven::VersionRange as ForgeVersionRange;
use semver::VersionReq as FabricVersionRange;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub enum SingleVersion {
    Fabric(FabricVersion),
    Forge(ForgeVersion),
}

#[derive(Debug, Clone, Deserialize)]
pub enum ManyVersions {
    Fabric(FabricVersionRange),
    Forge(ForgeVersionRange),
}
