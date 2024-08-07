use std::{
    collections::{HashMap, HashSet},
    ffi::{OsStr, OsString},
    fs,
    path::Path,
};

use anyhow::Context;
use jars::{jar, Jar, JarOptionBuilder};

use super::config::Config;

pub struct Pool {
    config: Config,
    /// mod slug - required versions
    remotes: HashSet<String>,
    locals: HashMap<OsString, Jar>,
}

impl Pool {
    pub fn new(dir_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let metadata = fs::metadata(&dir_path).context("Getting metadata")?;
        anyhow::ensure!(
            metadata.is_dir(),
            "The provided path should point to a directory"
        );

        let directory = fs::read_dir(&dir_path).context("Failed to read directory")?;
        let mut entries = directory
            .map(|entry| entry.map(|e| (e.file_name(), e.file_type())))
            .collect::<Result<HashMap<_, _>, _>>()
            .context("Collecting entries")?;

        // Check if `config.toml` file exists
        let config = {
            let (path, file_type) = entries
                .remove_entry(OsStr::new("config.toml"))
                .context("No `config.toml` file present in the pool")?;

            let file_type = file_type.context("Can't get metadata for `config.toml`")?;

            anyhow::ensure!(
                file_type.is_file(),
                "`config.toml` is expected to be a file"
            );

            Config::from_toml(dir_path.as_ref().join(path)).context("Reading `config.toml`")?
        };

        // Check if `remotes.json` file exists
        let remotes = {
            let (path, file_type) = entries
                .remove_entry(OsStr::new("remotes.json"))
                .context("No `remotes` directory present in the pool")?;

            let file_type = file_type.context("Can't get metadata for `remotes`")?;

            anyhow::ensure!(
                file_type.is_file(),
                "`remotes.json` is expected to be a file"
            );

            let content = fs::read_to_string(dir_path.as_ref().join(path))
                .context("Reading `remotes.json`")?;

            serde_json::from_str(&content).context("Deserializing `remotes.json`")?
        };

        // Check if `locals` directory exists
        let locals = {
            let (path, file_type) = entries
                .remove_entry(OsStr::new("locals"))
                .context("No `locals` directory present in the pool")?;

            let file_type = file_type.context("Can't get metadata for `locals`")?;

            anyhow::ensure!(file_type.is_dir(), "`locals` is expected to be a file");

            fs::read_dir(dir_path.as_ref().join(path))
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
}
