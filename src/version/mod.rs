pub mod maven;

use maven::Version as ForgeVersion;
use semver::Version as FabricVersion;

use maven::VersionRange as ForgeVersionRange;
use semver::VersionReq as FabricVersionRange;

#[derive(Debug, Clone)]
pub enum SingleVersion {
    Fabric(FabricVersion),
    Forge(ForgeVersion),
}

#[derive(Debug, Clone)]
pub enum ManyVersions {
    Fabric(FabricVersionRange),
    Forge(ForgeVersionRange),
}
