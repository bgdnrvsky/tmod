use semver::Version;
use serde::Deserialize;
use strum::{Display, EnumString};

#[derive(Debug, Clone, EnumString, Deserialize, Display)]
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
