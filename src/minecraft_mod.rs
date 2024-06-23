use anyhow::anyhow;
use std::collections::HashMap;
use std::path::Path;

use crate::{
    loader::Loaders,
    version::{MultiVersion, SingleVersion},
};
use anyhow::Context;
use jars::{jar, Jar, JarOptionBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModDep {
    #[serde(rename = "modId")]
    id: String,
    #[serde(rename = "versionRange")]
    versions: MultiVersion,
    mandatory: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModIncomp {
    id: String,
    versions: MultiVersion,
}

// TODO: Extract minecraft and forge from dependencies and make it a separate field
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
            dependencies: HashMap<String, Vec<ForgeModDep>>,
        }

        // NOTE: Might also include `displayName`
        #[derive(Debug, Deserialize)]
        struct ModInfo {
            #[serde(rename = "modId")]
            mod_id: String,
            version: crate::version::maven::Version,
        }

        #[derive(Clone, Debug, Deserialize)]
        struct ForgeModDep {
            #[serde(rename = "modId")]
            id: String,
            #[serde(rename = "versionRange")]
            versions: crate::version::maven::VersionRange,
            mandatory: bool,
        }

        impl From<ForgeModDep> for ModDep {
            fn from(forge_dep: ForgeModDep) -> Self {
                Self {
                    id: forge_dep.id,
                    versions: MultiVersion::Forge(forge_dep.versions),
                    mandatory: true,
                }
            }
        }

        let forge_toml = toml::from_str::<ForgeToml>(&String::from_utf8_lossy(content))
            .map_err(|e| anyhow!("Error while deserializing toml file META-INF/mods.toml: {e}"))?;

        let mod_info = forge_toml.mods.into_iter().next().context(
            "The `mods` array in META-INF/mods.toml file \
                     is expected to have at least one (probably the only) entry",
        )?;
        let mut all_dependencies = forge_toml.dependencies;
        let mod_id = mod_info.mod_id;
        let mod_dependencies: Option<Vec<ModDep>> = all_dependencies
            .remove(&mod_id)
            .map(|deps| deps.into_iter().map(ModDep::from).collect());

        Ok(Self {
            // TODO: Ignore dependencies that are not needed for client
            dependencies: mod_dependencies,
            id: mod_id,
            version: SingleVersion::Forge(mod_info.version),
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
            version: crate::version::fabric::Version,
            depends: HashMap<String, crate::version::fabric::VersionReq>,
            breaks: HashMap<String, crate::version::fabric::VersionReq>,
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
                        versions: MultiVersion::Fabric(versions),
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
                    .map(|(id, versions)| ModIncomp {
                        id,
                        versions: MultiVersion::Fabric(versions),
                    })
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

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn version(&self) -> &SingleVersion {
        &self.version
    }

    pub fn dependencies(&self) -> Option<&Vec<ModDep>> {
        self.dependencies.as_ref()
    }

    pub fn incompatibilities(&self) -> Option<&Vec<ModIncomp>> {
        self.incompatibilities.as_ref()
    }
}
