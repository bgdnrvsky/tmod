use std::collections::HashMap;

use anyhow::Context;
use jars::Jar;

use crate::version::{MultiVersion, SingleVersion};

pub mod fabric;
pub mod forge;

#[derive(Debug, Clone)]
pub enum JarMod {
    Fabric(fabric::FabricMod),
    Forge(forge::ForgeMod),
}

#[allow(unused)]
impl JarMod {
    pub fn name(&self) -> &str {
        match self {
            Self::Fabric(the_mod) => the_mod.slug(),
            Self::Forge(the_mod) => the_mod.slug(),
        }
    }

    pub fn version(&self) -> SingleVersion {
        match self {
            Self::Fabric(the_mod) => the_mod.version().clone().into(),
            Self::Forge(the_mod) => the_mod.version().clone().into(),
        }
    }

    pub fn minecraft_version(&self) -> MultiVersion {
        match self {
            Self::Fabric(the_mod) => the_mod.minecraft_version_needed().clone().into(),
            Self::Forge(the_mod) => the_mod.minecraft_version_needed().clone().into(),
        }
    }

    pub fn loader_version(&self) -> MultiVersion {
        match self {
            Self::Fabric(the_mod) => the_mod.loader_version_needed().clone().into(),
            Self::Forge(the_mod) => the_mod.loader_version_needed().clone().into(),
        }
    }

    pub fn dependencies(&self) -> HashMap<&str, MultiVersion> {
        match self {
            Self::Fabric(the_mod) => the_mod
                .dependencies()
                .iter()
                .map(|(slug, req)| (slug.as_str(), req.clone().into()))
                .collect(),
            Self::Forge(the_mod) => the_mod
                .dependencies()
                .iter()
                .map(|(slug, req)| (slug.as_str(), req.clone().into()))
                .collect(),
        }
    }

    pub fn incompatibilities(&self) -> HashMap<&str, MultiVersion> {
        match self {
            Self::Fabric(the_mod) => the_mod
                .dependencies()
                .iter()
                .map(|(slug, req)| (slug.as_str(), req.clone().into()))
                .collect(),
            Self::Forge(_) => HashMap::with_capacity(0),
        }
    }
}

impl TryFrom<Jar> for JarMod {
    type Error = anyhow::Error;

    fn try_from(jar: Jar) -> Result<Self, Self::Error> {
        if jar.files.contains_key("META-INF/mods.toml") {
            forge::ForgeMod::try_from(jar)
                .context("Reading Forge jar mod")
                .map(Self::Forge)
        } else if jar.files.contains_key("fabric.mod.json") {
            fabric::FabricMod::try_from(jar)
                .context("Reading Fabric jar mod")
                .map(Self::Fabric)
        } else {
            anyhow::bail!("No loader kind predicted");
        }
    }
}
