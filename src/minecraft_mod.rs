use anyhow::anyhow;
use std::collections::HashMap;
use std::{path::Path, str::FromStr};

use crate::{
    loader::Loaders,
    version::{ManyVersions, SingleVersion},
};
use anyhow::Context;
use jars::{jar, Jar, JarOptionBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ModDep {
    #[serde(rename = "modId")]
    id: String,
    #[serde(rename = "versionRange")]
    versions: ManyVersions,
    mandatory: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ModIncomp {
    id: String,
    versions: ManyVersions,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Mod {
    id: String,
    version: SingleVersion,
    dependencies: Option<Vec<ModDep>>,
    incompatibilities: Option<Vec<ModIncomp>>,
}

impl Mod {
    fn load_forge(jar: Jar) -> anyhow::Result<Self> {
        let content = jar
            .files
            .get("META-INF/mods.toml")
            .context("No META-INF/mods.toml in jar file while processing forge mod")?;

        #[derive(Debug, Deserialize)]
        /// META-INF/mods.toml file
        struct ForgeToml {
            mods: [ModInfo; 1],
            dependencies: HashMap<String, Vec<ModDep>>,
        }

        #[derive(Debug, Deserialize)]
        struct ModInfo {
            #[serde(rename = "modId")]
            mod_id: String,
            version: SingleVersion,
        }

        let forge_toml = toml::from_str::<ForgeToml>(&String::from_utf8_lossy(content))
            .map_err(|e| anyhow!("Error while deserializing toml file META-INF/mods.toml: {e}"))?;

        let mod_info = forge_toml.mods.into_iter().next().context(
            "The `mods` array in META-INF/mods.toml file \
                     is expected to have at least one (probably the only) entry",
        )?;
        let dependencies = forge_toml.dependencies;
        let id = mod_info.mod_id;

        Ok(Self {
            dependencies: dependencies.get(&id).map(|slice| slice.to_vec()),
            id,
            version: mod_info.version,
            incompatibilities: None,
        })
    }

    fn load_fabric(jar: Jar) -> anyhow::Result<Self> {
        let content = jar
            .files
            .get("fabric.mod.json")
            .context("No fabric.mod.json in jar file while processing fabric mod")?;

        #[derive(Debug, Deserialize)]
        struct FabricJson {
            id: String,
            version: semver::Version,
            depends: HashMap<String, ManyVersions>,
            breaks: HashMap<String, ManyVersions>,
        }

        let fabric_json = serde_json::from_slice::<FabricJson>(content)?;
        let dependencies: Option<Vec<ModDep>> = if fabric_json.depends.is_empty() {
            None
        } else {
            Some(
                fabric_json
                    .depends
                    .into_iter()
                    .map(|(id, versions)| ModDep {
                        id,
                        versions,
                        mandatory: true,
                    })
                    .collect::<Vec<_>>(),
            )
        };
        let incompatibilities: Option<Vec<ModIncomp>> = if fabric_json.breaks.is_empty() {
            None
        } else {
            Some(
                fabric_json
                    .breaks
                    .into_iter()
                    .map(|(id, versions)| ModIncomp { id, versions })
                    .collect(),
            )
        };

        Ok(Self {
            id: fabric_json.id,
            version: SingleVersion::Fabric(fabric_json.version),
            dependencies,
            incompatibilities,
        })
    }

    pub fn from_jar(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let jar = jar(path, JarOptionBuilder::builder().keep_meta_info().build())?;
        let loader_kind =
            Self::predict_loader(&jar).context("No loader kind found, check jar file!")?;

        match loader_kind {
            Loaders::Forge | Loaders::NeoForge => Self::load_forge(jar),
            Loaders::Fabric | Loaders::Quilt => Self::load_fabric(jar),
        }
    }

    pub fn predict_loader(jar: &Jar) -> Option<Loaders> {
        if jar.files.contains_key("META-INF/mods.toml") {
            Some(Loaders::Forge)
        } else if jar.files.contains_key("fabric.mod.json") {
            Some(Loaders::Fabric)
        } else {
            None
        }
    }
}
