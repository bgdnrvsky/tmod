use std::collections::HashMap;

use anyhow::Context;
use jars::Jar;
use serde::Deserialize;
use serde_with::DeserializeFromStr;
use strum::EnumString;

#[derive(Debug, Clone)]
pub struct ForgeMod {
    pub slug: String,
    pub version: String,
    pub loader_version_needed: Option<String>,
    pub minecraft_version_needed: Option<String>,
    /// Key: mod slug
    pub dependencies: HashMap<String, String>,
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
            .filter(|dependency| dependency.mandatory)
            .filter(|dependency| match &dependency.side {
                Some(side) => side.is_needed_for_client(),
                None => true,
            })
            .map(|dependency| (dependency.id, dependency.versions))
            .collect::<HashMap<_, _>>();

        let loader_version_needed = dependencies.remove("forge");

        let minecraft_version_needed = dependencies.remove("minecraft");

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
    version: String,
}

#[derive(Clone, Debug, Deserialize)]
struct ForgeModDep {
    #[serde(rename = "modId")]
    id: String,
    #[serde(rename = "versionRange")]
    versions: String,
    side: Option<Side>,
    mandatory: bool,
}

/// META-INF/mods.toml file
#[derive(Debug, Deserialize)]
struct ForgeToml {
    mods: [ModInfo; 1],
    #[serde(default)]
    dependencies: HashMap<String, Vec<ForgeModDep>>,
}
