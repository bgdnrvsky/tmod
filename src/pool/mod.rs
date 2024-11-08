pub mod config;
pub mod loader;

use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{BufReader, Write},
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
    pub manually_added: HashSet<String>,
    pub locks: HashMap<String, DepInfo>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DepInfo {
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub dependencies: Vec<String>,
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

    pub fn read(dir_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        if !dir_path.as_ref().try_exists().is_ok_and(|exists| exists) {
            anyhow::bail!("The pool '{}' doesnt exist!", dir_path.as_ref().display());
        }

        anyhow::ensure!(
            fs::metadata(&dir_path)?.is_dir(),
            "The provided path should point to a directory"
        );

        let mut entries = fs::read_dir(&dir_path)
            .context("Failed to read directory")?
            .collect::<Result<Vec<_>, _>>()?;

        let mut find_entry = |filename: &'static str| {
            entries
                .iter()
                .position(|f| f.file_name() == filename)
                .with_context(|| format!("No `{filename}` present in the pool"))
                .map(|idx| entries.swap_remove(idx))
        };

        // Check if `config.toml` file exists
        let config = {
            let file = find_entry("config.toml")?;

            anyhow::ensure!(
                file.metadata()?.is_file(),
                "`config.toml` is expected to be a file"
            );

            Config::from_toml(file.path()).context("Deserializing `config.toml`")?
        };

        // Check if `Tmod.json` file exists
        let manually_added = {
            let file = find_entry("Tmod.json")?;

            anyhow::ensure!(
                file.metadata()?.is_file(),
                "`Tmod.json` is expected to be a file"
            );

            let file = File::open(file.path()).context("Reading `Tmod.json`")?;
            let reader = BufReader::new(file);

            serde_json::from_reader(reader).context("Deserializing `Tmod.json`")?
        };

        let locks = {
            let file = find_entry("Tmod.lock")?;

            anyhow::ensure!(
                file.metadata()?.is_file(),
                "`Tmod.lock` is expected to be a file"
            );

            let content = fs::read_to_string(file.path()).context("Reading `Tmod.lock`")?;
            toml::from_str(&content).context("Deserializing `Tmod.lock`")?
        };

        Ok(Self {
            config,
            manually_added,
            locks,
            path: dir_path.as_ref().to_owned(),
        })
    }

    fn create_pool_dir(&self) -> anyhow::Result<()> {
        fs::DirBuilder::new()
            .recursive(true)
            .create(&self.path)
            .context("Creating dir")
    }

    fn write_config(&self) -> anyhow::Result<()> {
        let mut file = File::create(self.path.join("config.toml"))?;

        file.write_all(toml::to_string_pretty(&self.config)?.as_bytes())
            .context("Writing config")
    }

    fn write_remotes(&self) -> anyhow::Result<()> {
        // Writing tmod file
        let file = File::create(self.remotes_path())?;
        serde_json::to_writer_pretty(file, &self.manually_added).context("Writing remotes")?;

        // Writing lock file
        let mut file = File::create(self.locks_path())?;
        let content = toml::to_string_pretty(&self.locks).context("Serializing locks")?;

        file.write_all(content.as_bytes()).context("Writing locks")
    }

    pub fn save(&self) -> anyhow::Result<()> {
        self.create_pool_dir()
            .and_then(|_| Self::write_config(self))
            .and_then(|_| Self::write_remotes(self))
            .context("Saving pool")
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

    pub fn add_to_remotes(&mut self, the_mod: &SearchedMod, manual: bool) -> anyhow::Result<()> {
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
            dependencies: relations
                .iter()
                .map(|the_mod| the_mod.slug.clone())
                .collect(),
        };

        if manual {
            self.manually_added.insert(the_mod.slug.to_string());
        }

        for relation in relations {
            self.add_to_remotes(&relation, false)?;
        }

        self.locks.insert(the_mod.slug.to_string(), dep_info);

        Ok(())
    }

    pub fn remove_mod(&mut self, name: &str) -> bool {
        self.remove_from_remotes(name)
    }

    /// Returns whether the remote mod was already present
    pub fn remove_from_remotes(&mut self, name: &str) -> bool {
        self.manually_added.remove(name) && self.locks.remove(name).is_some()
    }

    pub fn root_path(&self) -> &Path {
        &self.path
    }

    pub fn remotes_path(&self) -> PathBuf {
        self.root_path().join("Tmod.json")
    }

    pub fn locks_path(&self) -> PathBuf {
        self.root_path().join("Tmod.lock")
    }
}
