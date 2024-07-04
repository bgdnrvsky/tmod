use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, ReadDir},
    path::PathBuf,
};

use anyhow::Context;

use crate::config::Config;

#[derive(Debug)]
pub struct Pool {
    config: Config,
    remotes: ReadDir,
    locals: ReadDir,
}

impl Pool {
    pub fn new(path: PathBuf) -> anyhow::Result<Self> {
        // Check that the path is a directory
        if !fs::metadata(&path).context("Getting metadata")?.is_dir() {
            return Err(anyhow::anyhow!(
                "The provided path should point to a directory"
            ));
        }

        let directory = fs::read_dir(&path).context("Failed to read directory")?;
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

            if !file_type.is_file() {
                return Err(anyhow::anyhow!("`config.toml` is expected to be a file"));
            }

            Config::from_toml(path).context("Reading `config.toml`")?
        };

        // Check if `remotes` directory exists
        let remotes = {
            let (path, file_type) = entries
                .remove_entry(OsStr::new("remotes"))
                .context("No `remotes` directory present in the pool")?;

            let file_type = file_type.context("Can't get metadata for `remotes`")?;

            if !file_type.is_dir() {
                return Err(anyhow::anyhow!("`remotes` is expected to be a file"));
            }

            fs::read_dir(path).context("Reading `remotes` directory")?
        };

        // Check if `locals` directory exists
        let locals = {
            let (path, file_type) = entries
                .remove_entry(OsStr::new("locals"))
                .context("No `locals` directory present in the pool")?;

            let file_type = file_type.context("Can't get metadata for `locals`")?;

            if !file_type.is_dir() {
                return Err(anyhow::anyhow!("`locals` is expected to be a file"));
            }

            fs::read_dir(path).context("Reading `locals` directory")?
        };

        Ok(Self {
            config,
            remotes,
            locals,
        })
    }
}
