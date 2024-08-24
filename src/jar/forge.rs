use std::collections::HashMap;

use anyhow::Context;
use jars::Jar;
use serde::Deserialize;
use serde_with::DeserializeFromStr;
use strum::EnumString;

use crate::version::maven::{Version, VersionRange};

#[derive(Debug, Clone)]
pub struct ForgeMod {
    slug: String,
    version: Version,
    loader_version_needed: VersionRange,
    minecraft_version_needed: VersionRange,
    /// Key: mod slug
    dependencies: HashMap<String, VersionRange>,
}

impl ForgeMod {
    pub fn slug(&self) -> &str {
        &self.slug
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn loader_version_needed(&self) -> &VersionRange {
        &self.loader_version_needed
    }

    pub fn minecraft_version_needed(&self) -> &VersionRange {
        &self.minecraft_version_needed
    }

    pub fn dependencies(&self) -> &HashMap<String, VersionRange> {
        &self.dependencies
    }
}

impl TryFrom<&Jar> for ForgeMod {
    type Error = anyhow::Error;

    fn try_from(jar: &Jar) -> Result<Self, Self::Error> {
        let content = jar
            .files
            .get("META-INF/mods.toml")
            .context("No META-INF/mods.toml in jar file while processing forge mod")?;

        let mut forge_toml = toml::from_str::<ForgeToml>(&String::from_utf8_lossy(content))
            .context("Deserializing toml file META-INF/mods.toml")?;

        let mod_info = forge_toml
            .mods
            .into_iter()
            .next()
            .expect("mods array contains only one element");

        let slug = mod_info.slug;
        let mod_deps = forge_toml
            .dependencies
            .remove(&slug)
            .unwrap_or_else(Vec::new);

        let mut dependencies = mod_deps
            .into_iter()
            .filter(|dependency| dependency.side.is_needed_for_client())
            .map(|dependency| (dependency.id, dependency.versions))
            .collect::<HashMap<_, _>>();

        let loader_version_needed = dependencies
            .remove("forge")
            .context("Jar mod config didn't specify the required loader version range")?;

        let minecraft_version_needed = dependencies
            .remove("minecraft")
            .context("Jar mod config didn't specify the required minecraft version range")?;

        Ok(Self {
            slug,
            version: mod_info.version,
            loader_version_needed,
            minecraft_version_needed,
            dependencies,
        })
    }
}

impl TryFrom<Jar> for ForgeMod {
    type Error = anyhow::Error;

    fn try_from(jar: Jar) -> Result<Self, Self::Error> {
        Self::try_from(&jar)
    }
}

#[derive(Debug, Clone, EnumString, DeserializeFromStr, PartialEq, Eq)]
#[strum(ascii_case_insensitive)]
enum Side {
    Both,
    Client,
    Server,
}

impl Side {
    fn is_needed_for_client(&self) -> bool {
        matches!(self, Self::Both | Self::Client)
    }
}

#[derive(Debug, Deserialize)]
struct ModInfo {
    #[serde(rename = "modId")]
    slug: String,
    #[allow(unused)]
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    version: crate::version::maven::Version,
}

#[derive(Clone, Debug, Deserialize)]
struct ForgeModDep {
    #[serde(rename = "modId")]
    id: String,
    #[serde(rename = "versionRange")]
    versions: crate::version::maven::VersionRange,
    side: Side,
    // NOTE: What to do with it ?
    #[allow(unused)]
    mandatory: bool,
}

/// META-INF/mods.toml file
#[derive(Debug, Deserialize)]
struct ForgeToml {
    mods: [ModInfo; 1],
    dependencies: HashMap<String, Vec<ForgeModDep>>,
}
