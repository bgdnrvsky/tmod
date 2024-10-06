use std::collections::HashMap;

use anyhow::Context;
use jars::Jar;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct FabricMod {
    pub slug: String,
    pub version: String,
    pub loader_version_needed: Option<String>,
    pub minecraft_version_needed: Option<String>,
    // Key: mod slug
    pub dependencies: HashMap<String, String>,
    // Key: mod slug
    pub incompatibilities: HashMap<String, String>,
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
        dependencies.retain(|k, _| !k.starts_with("fabric"));

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
