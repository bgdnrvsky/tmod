use std::collections::HashMap;

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
    fn name(&self) -> &str {
        match self {
            JarMod::Fabric(the_mod) => the_mod.slug(),
            JarMod::Forge(the_mod) => the_mod.slug(),
        }
    }

    fn version(&self) -> SingleVersion {
        match self {
            JarMod::Fabric(the_mod) => SingleVersion::Fabric(the_mod.version().clone()),
            JarMod::Forge(the_mod) => SingleVersion::Forge(the_mod.version().clone()),
        }
    }

    fn minecraft_version(&self) -> MultiVersion {
        match self {
            JarMod::Fabric(the_mod) => {
                MultiVersion::Fabric(the_mod.minecraft_version_needed().clone())
            }
            JarMod::Forge(the_mod) => {
                MultiVersion::Forge(the_mod.minecraft_version_needed().clone())
            }
        }
    }

    fn loader_version(&self) -> MultiVersion {
        match self {
            JarMod::Fabric(the_mod) => {
                MultiVersion::Fabric(the_mod.loader_version_needed().clone())
            }
            JarMod::Forge(the_mod) => MultiVersion::Forge(the_mod.loader_version_needed().clone()),
        }
    }

    fn dependencies(&self) -> HashMap<&str, MultiVersion> {
        // self.dependencies().into_iter().map(|(slug, ver)| (slug, MutliVersion::
        match self {
            JarMod::Fabric(the_mod) => the_mod
                .dependencies()
                .iter()
                .map(|(slug, req)| (slug.as_str(), MultiVersion::Fabric(req.clone())))
                .collect(),
            JarMod::Forge(the_mod) => the_mod
                .dependencies()
                .iter()
                .map(|(slug, req)| (slug.as_str(), MultiVersion::Forge(req.clone())))
                .collect(),
        }
    }

    fn incompabilites(&self) -> HashMap<&str, MultiVersion> {
        match self {
            JarMod::Fabric(the_mod) => the_mod
                .dependencies()
                .iter()
                .map(|(slug, req)| (slug.as_str(), MultiVersion::Fabric(req.clone())))
                .collect(),
            JarMod::Forge(_) => HashMap::with_capacity(0),
        }
    }
}
