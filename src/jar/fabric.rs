use std::collections::HashMap;

use anyhow::Context;
use jars::Jar;
use serde::Deserialize;

use crate::version::fabric::{Version, VersionReq};

#[derive(Debug, Clone)]
pub struct FabricMod {
    slug: String,
    version: Version,
    loader_version_needed: VersionReq,
    minecraft_version_needed: VersionReq,
    // Key: mod slug
    dependencies: HashMap<String, VersionReq>,
    // Key: mod slug
    incompatibilities: HashMap<String, VersionReq>,
}

impl FabricMod {
    pub fn slug(&self) -> &str {
        &self.slug
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn loader_version_needed(&self) -> &VersionReq {
        &self.loader_version_needed
    }

    pub fn minecraft_version_needed(&self) -> &VersionReq {
        &self.minecraft_version_needed
    }

    pub fn dependencies(&self) -> &HashMap<String, VersionReq> {
        &self.dependencies
    }

    pub fn incompatibilities(&self) -> &HashMap<String, VersionReq> {
        &self.incompatibilities
    }
}

impl TryFrom<&Jar> for FabricMod {
    type Error = anyhow::Error;

    fn try_from(jar: &Jar) -> Result<Self, Self::Error> {
        let content = jar
            .files
            .get("fabric.mod.json")
            .context("No fabric.mod.json in jar file while processing fabric mod")?;

        let fabric_json = serde_json::from_slice::<FabricJson>(content)?;
        let mut dependencies = fabric_json.depends;

        let incompatibilities = fabric_json.breaks;

        let loader_version_needed = dependencies
            .remove("fabricloader")
            .unwrap_or_else(VersionReq::any);

        let minecraft_version_needed = dependencies
            .remove("minecraft")
            .unwrap_or_else(VersionReq::any);

        // The JAR may contain some dependencies that are not remote,
        // so if in the future we try to build a tree, for example,
        // the searcher will not succeed to find the mod by slug online and error
        dependencies = dependencies
            .into_iter()
            .filter_map(|(key, value)| {
                if key.starts_with("fabric") {
                    None
                } else {
                    Some((key, value))
                }
            })
            .collect();

        // Remove more internal dependencies
        dependencies.remove("java");

        // Change some known edgecases where jar developers have made a mistake
        if let Some(version) = dependencies.remove("fzzy_core") {
            dependencies.insert(String::from("fzzy-core"), version);
        }

        Ok(Self {
            slug: fabric_json.id,
            version: fabric_json.version,
            loader_version_needed,
            minecraft_version_needed,
            dependencies,
            incompatibilities,
        })
    }
}

impl TryFrom<Jar> for FabricMod {
    type Error = anyhow::Error;

    fn try_from(jar: Jar) -> Result<Self, Self::Error> {
        Self::try_from(&jar)
    }
}

#[derive(Debug, Deserialize)]
struct FabricJson {
    id: String,
    version: Version,
    #[serde(default)]
    depends: HashMap<String, VersionReq>,
    #[serde(default)]
    breaks: HashMap<String, VersionReq>,
}
