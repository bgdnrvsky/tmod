pub mod config;
mod io;
pub mod loader;

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::fetcher::{mod_search::search_mod::SearchedMod, SEARCHER};

use config::Config;

pub struct Pool {
    pub path: PathBuf,
    pub config: Config,
    pub manually_added: BTreeSet<String>,
    pub locks: BTreeMap<String, DepInfo>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DepInfo {
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "std::ops::Not::not")] // Don't serialize if false
    #[serde(default)]
    pub client_only: bool,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    #[serde(default)]
    pub dependencies: BTreeSet<String>,
}

impl Pool {
    pub fn init(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let config = Config::init()?;

        let pool = Self {
            path: path.as_ref().to_owned(),
            config,
            locks: Default::default(),
            manually_added: Default::default(),
        };

        pool.save().map(|_| pool)
    }

    pub fn is_compatible(&self, the_mod: &SearchedMod) -> anyhow::Result<bool> {
        let file = SEARCHER.get_specific_mod_file(the_mod, &self.config, None);

        if file.is_err() {
            return Ok(false);
        }

        for inc_id in file
            .unwrap()
            .relations
            .iter()
            .filter(|dep| dep.relation.is_incompatible())
            .map(|dep| dep.id)
        {
            // Get all incompatibilities for the file, and check if it is present in the pool
            let inc = SEARCHER
                .search_mod_by_id(inc_id)
                .with_context(|| format!("Couldn't find the incompatibility id ({inc_id})"))?;

            for remote_slug in self.manually_added.iter() {
                if inc.slug == *remote_slug {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    pub fn add_to_remotes(
        &mut self,
        the_mod: &SearchedMod,
        client_only: bool,
        manual: bool,
    ) -> anyhow::Result<()> {
        if !self.is_compatible(the_mod)? {
            anyhow::bail!(
                "The mod {slug} is not compatible with the pool!",
                slug = the_mod.slug
            );
        }

        let file = SEARCHER.get_specific_mod_file(the_mod, &self.config, None)?;
        let relations = file
            .relations
            .into_iter()
            .map(|relation| {
                SEARCHER.search_mod_by_id(relation.id).with_context(|| {
                    format!(
                        "Searching a relation id={} while adding mod '{}'",
                        relation.id, the_mod.slug
                    )
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()
            .with_context(|| format!("Searching relations of the mod '{}'", the_mod.slug))?;

        let dep_info = DepInfo {
            timestamp: file.date,
            client_only,
            dependencies: relations
                .iter()
                .map(|the_mod| the_mod.slug.clone())
                .collect(),
        };

        if manual {
            self.manually_added.insert(the_mod.slug.to_string());
        }

        for relation in relations {
            self.add_to_remotes(&relation, client_only, false)?;
        }

        self.locks.insert(the_mod.slug.to_string(), dep_info);

        Ok(())
    }

    // Returns whether the remote mod was already present
    pub fn remove_mod(&mut self, name: &str) -> bool {
        self.manually_added.remove(name) && self.locks.remove(name).is_some()
    }
}
