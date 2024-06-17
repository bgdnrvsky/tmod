use std::{path::Path, str::FromStr};

use crate::{loader::Loaders, version::SingleVersion};
use anyhow::Context;
use jars::{jar, Jar, JarOptionBuilder};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use toml::{Table, Value as TomlValue};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ModDep {
    #[serde(rename = "modId")]
    id: String,
    #[serde(rename = "versionRange")]
    versions: String,
    mandatory: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ModIncomp {
    id: String,
    versions: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Mod {
    id: String,
    version: SingleVersion,
    dependencies: Vec<ModDep>,
    incompatibilities: Option<Vec<ModIncomp>>,
}

impl Mod {
    fn load_forge(jar: Jar) -> anyhow::Result<Self> {
        let content = jar
            .files
            .get("META-INF/mods.toml")
            .context("No META-INF/mods.toml in jar file while processing forge mod")?;

        let table = toml::from_str::<Table>(&String::from_utf8(content.to_vec())?)?;

        let mod_info = table
            .get("mods")
            .context("Forge mod spec is expected to have an array `mods`")?
            .get(0)
            .context(
                "Forge `mods` array is expected to have at least (probably the only) one element",
            )?
            .as_table()
            .context("The entry in `mods` array should be a table")?;

        let mod_id = mod_info
            .get("modId")
            .context("No key `modId` in forge mod's spec")?
            .as_str()
            .context("The key `modId` was not a string")?;

        let mod_version = mod_info
            .get("version")
            .context("No key `version` in forge mod's spec")?
            .as_str()
            .context("The key `version` was not a string")?;

        let dependencies: anyhow::Result<Vec<ModDep>> = table
            .get("dependencies")
            .context("Forge's mod spec didn't have the key `dependencies`")?
            .as_table()
            .context("The key `dependencies` in forge mod's spec is expected to be a table")?
            .get(mod_id)
            .context("Dependencies table in forge mod's spec is expected to have a key matching the mod id")?
            .as_array()
            .context("Dependencies in forge mod's spec weren't in an array")?
            .iter()
            .map(|value| TomlValue::try_into(value.clone()).context("Couldn't transform dependency entry"))
            .collect();

        Ok(Self {
            id: mod_id.to_owned(),
            version: SingleVersion::Forge(crate::version::maven::Version::from_str(mod_version)?),
            dependencies: dependencies?,
            incompatibilities: None,
        })
    }

    fn load_fabric(jar: Jar) -> anyhow::Result<Self> {
        let content = jar
            .files
            .get("fabric.mod.json")
            .context("No fabric.mod.json in jar file while processing fabric mod")?;

        let json = serde_json::from_slice::<JsonValue>(content)?;
        let object = json
            .as_object()
            .context("fabric.mod.json file is expected to be an object (map)")?;

        let mod_id = object
            .get("id")
            .context("Fabric mod's spec didn't contain the key `id`")?
            .as_str()
            .context("Fabric mod's id wasn't a string")?;

        let mod_version = object
            .get("version")
            .context("Fabric mod's spec didn't contain the key `version`")?
            .as_str()
            .context("Fabric mod's version wasn't a string")?;

        let dependencies: Vec<ModDep> = object
            .get("depends")
            .context("No key `depends` in fabric mod's spec")?
            .as_object()
            .context("The key `depends` wasn't an object (map)")?
            .into_iter()
            .map(|(id, versions)| ModDep {
                id: id.to_string(),
                versions: versions.to_string(),
                mandatory: true,
            })
            .collect();

        let incompatibilities: Option<Vec<ModIncomp>> = object
            .get("breaks")
            .and_then(|value| value.as_object())
            .map(|object| {
                object.into_iter().map(|(id, versions)| ModIncomp {
                    id: id.to_string(),
                    versions: versions.to_string(),
                })
            })
            .map(Iterator::collect);

        Ok(Self {
            id: mod_id.to_string(),
            version: SingleVersion::Fabric(semver::Version::from_str(mod_version)?),
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