use std::collections::HashMap;

use anyhow::Context;
use jars::Jar;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct FabricMod {
    slug: String,
    version: String,
    loader_version_needed: Option<String>,
    minecraft_version_needed: Option<String>,
    // Key: mod slug
    dependencies: HashMap<String, String>,
    // Key: mod slug
    incompatibilities: HashMap<String, String>,
}

impl FabricMod {
    pub fn slug(&self) -> &str {
        &self.slug
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn loader_version_needed(&self) -> Option<&str> {
        self.loader_version_needed.as_deref()
    }

    pub fn minecraft_version_needed(&self) -> Option<&str> {
        self.minecraft_version_needed.as_deref()
    }

    pub fn dependencies(&self) -> &HashMap<String, String> {
        &self.dependencies
    }

    pub fn incompatibilities(&self) -> &HashMap<String, String> {
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

        let FabricJson {
            slug,
            version,
            mut dependencies,
            incompatibilities,
        } = serde_json::from_slice::<FabricJson>(content)?;

        let loader_version_needed = dependencies.remove("fabricloader");
        let minecraft_version_needed = dependencies.remove("minecraft");

        // The JAR may contain some dependencies that are not remote,
        // so if in the future we try to build a tree, for example,
        // the searcher will not succeed to find the mod by slug online
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
            slug,
            version,
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
    #[serde(rename = "id")]
    slug: String,
    version: String,
    #[serde(default)]
    #[serde(rename = "depends")]
    dependencies: HashMap<String, String>,
    #[serde(default)]
    #[serde(rename = "breaks")]
    incompatibilities: HashMap<String, String>,
}
