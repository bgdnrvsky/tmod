use std::collections::HashMap;

use anyhow::Context;
use jars::Jar;

use crate::version::{MultiVersion, SingleVersion};

pub mod fabric;
pub mod forge;

#[derive(Debug, Clone)]
enum JarModType {
    Fabric(fabric::FabricMod),
    Forge(forge::ForgeMod),
}

pub struct JarMod {
    r#type: JarModType,
    zip: Jar,
}

impl JarMod {
    pub fn name(&self) -> &str {
        self.r#type.name()
    }

    pub fn version(&self) -> SingleVersion {
        self.r#type.version()
    }

    pub fn minecraft_version(&self) -> MultiVersion {
        self.r#type.minecraft_version()
    }

    pub fn loader_version(&self) -> MultiVersion {
        self.r#type.loader_version()
    }

    pub fn dependencies(&self) -> HashMap<&str, MultiVersion> {
        self.r#type.dependencies()
    }

    pub fn incompatibilities(&self) -> HashMap<&str, MultiVersion> {
        self.r#type.incompatibilities()
    }

    pub fn zip(&self) -> &Jar {
        &self.zip
    }
}

impl JarModType {
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

impl TryFrom<&Jar> for JarModType {
    type Error = anyhow::Error;

    fn try_from(jar: &Jar) -> Result<Self, Self::Error> {
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

impl TryFrom<Jar> for JarModType {
    type Error = anyhow::Error;

    fn try_from(jar: Jar) -> Result<Self, Self::Error> {
        Self::try_from(&jar)
    }
}

impl TryFrom<Jar> for JarMod {
    type Error = anyhow::Error;

    fn try_from(jar: Jar) -> Result<Self, Self::Error> {
        Ok(Self {
            r#type: JarModType::try_from(&jar)?,
            zip: jar,
        })
    }
}
