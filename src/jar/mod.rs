use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Context;
use jars::{jar, Jar, JarOption};

pub mod fabric;
pub mod forge;

#[derive(Debug, Clone)]
enum JarModType {
    Fabric(fabric::FabricMod),
    Forge(forge::ForgeMod),
}

pub struct JarMod {
    path: PathBuf,
    r#type: JarModType,
    zip: Jar,
}

impl JarMod {
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();

        if !path.extension().is_some_and(|ext| ext == "jar") {
            eprintln!(
                "WARNING: The file '{}' doesn't seem like a jar",
                path.display()
            );
        }

        let jar = jar(path, JarOption::default())
            .with_context(|| format!("Opening jar '{}'", path.display()))?;
        let r#type = JarModType::try_from(&jar)
            .with_context(|| format!("Reading jar '{}'", path.display()))?;

        Ok(Self {
            path: path.file_name().unwrap().into(),
            r#type,
            zip: jar,
        })
    }

    pub fn name(&self) -> &str {
        self.r#type.name()
    }

    pub fn version(&self) -> &str {
        self.r#type.version()
    }

    pub fn minecraft_version(&self) -> Option<&str> {
        self.r#type.minecraft_version()
    }

    pub fn loader_version(&self) -> Option<&str> {
        self.r#type.loader_version()
    }

    // TODO: Introduce dependencies substitutions and deletion for cli
    pub fn dependencies(&self) -> HashMap<&str, &str> {
        self.r#type.dependencies()
    }

    pub fn incompatibilities(&self) -> HashMap<&str, &str> {
        self.r#type.incompatibilities()
    }

    pub fn zip(&self) -> &Jar {
        &self.zip
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl JarModType {
    pub fn name(&self) -> &str {
        match self {
            Self::Fabric(the_mod) => &the_mod.slug,
            Self::Forge(the_mod) => &the_mod.slug,
        }
    }

    pub fn version(&self) -> &str {
        match self {
            Self::Fabric(the_mod) => &the_mod.version,
            Self::Forge(the_mod) => &the_mod.version,
        }
    }

    pub fn minecraft_version(&self) -> Option<&str> {
        match self {
            Self::Fabric(the_mod) => the_mod.minecraft_version_needed.as_deref(),
            Self::Forge(the_mod) => the_mod.minecraft_version_needed.as_deref(),
        }
    }

    pub fn loader_version(&self) -> Option<&str> {
        match self {
            Self::Fabric(the_mod) => the_mod.loader_version_needed.as_deref(),
            Self::Forge(the_mod) => the_mod.loader_version_needed.as_deref(),
        }
    }

    /// Returns a map of dependencies with their versions
    pub fn dependencies(&self) -> HashMap<&str, &str> {
        match self {
            Self::Fabric(the_mod) => the_mod
                .dependencies
                .iter()
                .map(|(slug, req)| (slug.as_str(), req.as_str()))
                .collect(),
            Self::Forge(the_mod) => the_mod
                .dependencies
                .iter()
                .map(|(slug, req)| (slug.as_str(), req.as_str()))
                .collect(),
        }
    }

    /// Returns a map of incompatibilities with their versions
    pub fn incompatibilities(&self) -> HashMap<&str, &str> {
        match self {
            Self::Fabric(the_mod) => the_mod
                .incompatibilities
                .iter()
                .map(|(slug, req)| (slug.as_str(), req.as_str()))
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
