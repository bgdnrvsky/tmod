pub mod config;
pub mod loader;

use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    fs,
    path::Path,
};

use anyhow::Context;
use jars::{jar, Jar, JarOptionBuilder};

use crate::fetcher::mod_search::search_mod::SearchedMod;

use config::Config;

pub struct Pool {
    config: Config,
    /// mod slug - required versions
    remotes: HashSet<String>,
    locals: HashMap<OsString, Jar>,
}

impl Pool {
    pub fn new(dir_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        anyhow::ensure!(
            fs::metadata(&dir_path)?.is_dir(),
            "The provided path should point to a directory"
        );

        let mut entries = fs::read_dir(&dir_path)
            .context("Failed to read directory")?
            .collect::<Result<Vec<_>, _>>()?;

        // Check if `config.toml` file exists
        let config = {
            let file = entries.swap_remove(
                entries
                    .iter()
                    .position(|f| f.file_name() == "config.toml")
                    .context("No `config.toml` present in the pool")?,
            );

            anyhow::ensure!(
                file.metadata()?.is_file(),
                "`config.toml` is expected to be a file"
            );

            Config::from_toml(file.path()).context("Deserializing `config.toml`")?
        };

        // Check if `remotes.json` file exists
        let remotes = {
            let file = entries.swap_remove(
                entries
                    .iter()
                    .position(|f| f.file_name() == "remotes.json")
                    .context("No `remotes.json` present in the pool")?,
            );

            anyhow::ensure!(
                file.metadata()?.is_file(),
                "`remotes.json` is expected to be a file"
            );

            let content = fs::read_to_string(file.path()).context("Reading `remotes.json`")?;

            serde_json::from_str(&content).context("Deserializing `remotes.json`")?
        };

        // Check if `locals` directory exists
        let locals = {
            let dir = entries.swap_remove(
                entries
                    .iter()
                    .position(|f| f.file_name() == "locals")
                    .context("No `locals` directory present in the pool")?,
            );

            anyhow::ensure!(
                dir.metadata()?.is_dir(),
                "`locals` is expected to be a directory"
            );

            fs::read_dir(dir.path())
                .context("Reading `locals` directory")?
                .map(|entry| entry.map(|entry| entry.file_name()))
                .map(|entry| {
                    entry.and_then(|file_name| {
                        jar(&file_name, JarOptionBuilder::default()).map(|jar| (file_name, jar))
                    })
                })
                .collect::<Result<HashMap<_, _>, _>>()?
        };

        Ok(Self {
            config,
            remotes,
            locals,
        })
    }

    pub fn remotes(&self) -> &HashSet<String> {
        &self.remotes
    }

    pub fn locals(&self) -> &HashMap<OsString, Jar> {
        &self.locals
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn add_to_remotes(&mut self, the_mod: &SearchedMod) -> anyhow::Result<()> {
        unimplemented!()
    }
}
