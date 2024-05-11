mod config;
mod fetchers;
mod loader;

use std::{collections::HashMap, path::Path};

use anyhow::{anyhow, Context};
use jars::{jar, Jar, JarOptionBuilder};
use loader::Loaders;
use serde::{Deserialize, Serialize};
use toml::{Table, Value};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ModDep {
    #[serde(rename = "modId")]
    id: String,
    #[serde(rename = "versionRange")]
    versions: String,
    mandatory: bool
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Mod {
    id: String,
    version: String, // TODO: Use custom version struct
    dependencies: Vec<ModDep>,
}

impl Mod {
    fn load_forge(jar: Jar) -> anyhow::Result<Self> {
        let content = jar.files.get("META-INF/mods.toml").ok_or(anyhow!(
            "No META-INF/mods.toml in jar file while processing forge mod"
        ))?;

        let table = toml::from_str::<Table>(&String::from_utf8(content.to_vec())?)?;

        let mod_info = table
            .get("mods")
            .ok_or(anyhow!(
                "Forge mod spec is expected to have an array `mods`"
            ))?
            .get(0)
            .ok_or(anyhow!(
                "Forge `mods` array is expected to have at least (probably the only) one element"
            ))?
            .as_table()
            .ok_or(anyhow!("The entry in `mods` array should be a table"))?;

        let mod_id = mod_info
            .get("modId")
            .ok_or(anyhow!("No key `modId` in forge mod's spec"))?
            .as_str()
            .ok_or(anyhow!("The key `modId` was not a string"))?;

        let mod_version = mod_info
            .get("version")
            .ok_or(anyhow!("No key `version` in forge mod's spec"))?
            .as_str()
            .ok_or(anyhow!("The key `version` was not a string"))?;

        let dependencies: anyhow::Result<Vec<ModDep>> =
            table.get("dependencies").ok_or(anyhow!("Forge's mod spec didn't have the key `dependencies`"))?
            .as_table()
            .ok_or(anyhow!("The key `dependencies` in forge mod's spec is expected to be a table"))?
            .get(mod_id)
            .ok_or(anyhow!("Dependencies table in forge mod's spec is expected to have a key matching the mod id"))?
            .as_array()
            .ok_or(anyhow!("Dependencies in forge mod's spec weren't in an array"))?
            .iter()
            .map(|value| Value::try_into(value.clone()).map_err(|_| anyhow!("error")))
            .collect();

        Ok(Self {
            id: mod_id.to_owned(),
            version: mod_version.to_owned(),
            dependencies: dependencies?,
        })
    }

    fn load_fabric(jar: Jar) -> anyhow::Result<Self> {
        todo!()
    }

    pub fn from_jar(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let jar = jar(path, JarOptionBuilder::builder().keep_meta_info().build())?;
        let loader_kind =
            Self::predict_loader(&jar).ok_or(anyhow!("No loader kind found, check jar file!"))?;

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

fn main() -> anyhow::Result<()> {
    let jar = jar(
        "btp.jar",
        JarOptionBuilder::builder().keep_meta_info().build(),
    )?;

    println!("{:#?}", Mod::load_forge(jar));

    Ok(())
}
