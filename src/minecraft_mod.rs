use crate::{
    loader::Loaders,
    version::{MultiVersion, SingleVersion},
};
use anyhow::anyhow;
use anyhow::Context;
use jars::{jar, Jar, JarOptionBuilder};
use serde::{Deserialize, Serialize};
use serde_with::DeserializeFromStr;
use std::collections::HashMap;
use std::path::Path;
use strum::EnumString;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModDepInfo {
    versions: MultiVersion,
    mandatory: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModIncomp {
    id: String,
    versions: MultiVersion,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Mod {
    id: String,
    version: SingleVersion,
    loader_version_needed: MultiVersion,
    minecraft_version_needed: MultiVersion,
    /// Key: name of the mod dep (slug), value: dep info
    dependencies: Option<HashMap<String, ModDepInfo>>,
    incompatibilities: Option<Vec<ModIncomp>>,
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
            side: Side,
            mandatory: bool,
        }

        let mut forge_toml = toml::from_str::<ForgeToml>(&String::from_utf8_lossy(content))
            .map_err(|e| anyhow!("Error while deserializing toml file META-INF/mods.toml: {e}"))?;

        let mod_info = forge_toml.mods.into_iter().next().context(
            "The `mods` array in META-INF/mods.toml file \
                     is expected to have at least one (probably the only) entry",
        )?;

        let mod_id = mod_info.mod_id;

        let mut dependencies = forge_toml.dependencies.remove(&mod_id).map(|dependencies| {
            dependencies
                .into_iter()
                .filter(|dependency| dependency.side.is_needed_for_client())
                .map(|dependency| {
                    (
                        dependency.id,
                        ModDepInfo {
                            versions: MultiVersion::Forge(dependency.versions),
                            mandatory: dependency.mandatory,
                        },
                    )
                })
                .collect::<HashMap<String, _>>()
        });

        let loader_version_needed = dependencies
            .as_mut()
            .and_then(|deps| deps.remove("forge"))
            .map(|dep| dep.versions)
            .context("Jar mod config didn't specify the required loader version range")?;

        let minecraft_version_needed = dependencies
            .as_mut()
            .and_then(|deps| deps.remove("minecraft"))
            .map(|dep| dep.versions)
            .context("Jar mod config didn't specify the required loader version range")?;

        Ok(Self {
            dependencies,
            id: mod_id,
            version: SingleVersion::Forge(mod_info.version),
            incompatibilities: None,
            loader_version_needed,
            minecraft_version_needed,
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
        let dependencies = if fabric_json.depends.is_empty() {
            None
        } else {
            Some(
                fabric_json
                    .depends
                    .into_iter()
                    .map(|(id, versions)| {
                        (
                            id,
                            ModDepInfo {
                                versions: MultiVersion::Fabric(versions),
                                mandatory: true,
                            },
                        )
                    })
                    .collect(),
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

    pub fn dependencies(&self) -> Option<&HashMap<String, ModDepInfo>> {
        self.dependencies.as_ref()
    }

    pub fn incompatibilities(&self) -> Option<&Vec<ModIncomp>> {
        self.incompatibilities.as_ref()
    }
}

#[cfg(test)]
mod side {
    use std::str::FromStr;

    use super::Side;

    #[test]
    fn from_str() {
        assert_eq!(Side::from_str("both"), Ok(Side::Both));
        assert_eq!(Side::from_str("BOTH"), Ok(Side::Both));

        assert_eq!(Side::from_str("client"), Ok(Side::Client));
        assert_eq!(Side::from_str("CLIENT"), Ok(Side::Client));

        assert_eq!(Side::from_str("server"), Ok(Side::Server));
        assert_eq!(Side::from_str("SERVER"), Ok(Side::Server));
    }

    #[test]
    fn is_needed() {
        assert!(Side::Both.is_needed_for_client());
        assert!(Side::Client.is_needed_for_client());
        assert!(!Side::Server.is_needed_for_client());
    }
}
