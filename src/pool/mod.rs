pub mod config;
pub mod loader;

use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    fs::{self, File},
    io::{BufReader, Write},
    path::{Path, PathBuf},
};

use anyhow::Context;
use jars::{jar, Jar, JarOptionBuilder};

use crate::fetcher::mod_search::search_mod::SearchedMod;

use config::Config;

pub struct Pool {
    path: PathBuf,
    config: Config,
    remotes: HashSet<String>,
    locals: HashMap<OsString, Jar>,
}

impl Pool {
    pub fn init(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let config = Config::init()?;

        let pool = Self {
            path: path.as_ref().to_owned(),
            config,
            remotes: Default::default(),
            locals: Default::default(),
        };

        pool.save().map(|_| pool)
    }

    pub fn new(dir_path: impl AsRef<Path>) -> anyhow::Result<Self> {
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

        // Check if `remotes.json` file exists
        let remotes = {
            let file = find_entry("remotes.json")?;

            anyhow::ensure!(
                file.metadata()?.is_file(),
                "`remotes.json` is expected to be a file"
            );

            let file = File::open(file.path()).context("Opening `remotes.json`")?;
            let reader = BufReader::new(file);

            serde_json::from_reader(reader).context("Deserializing `remotes.json`")?
        };

        // Check if `locals` directory exists
        let locals = {
            let dir = find_entry("locals")?;

            anyhow::ensure!(
                dir.metadata()?.is_dir(),
                "`locals` is expected to be a directory"
            );

            fs::read_dir(dir.path())
                .context("Reading `locals` directory")?
                .map(|entry| {
                    entry.and_then(|entry| {
                        jar(entry.path(), JarOptionBuilder::default())
                            .map(|jar| (entry.file_name(), jar))
                    })
                })
                .collect::<Result<HashMap<_, _>, _>>()?
        };

        Ok(Self {
            config,
            remotes,
            locals,
            path: dir_path.as_ref().to_owned(),
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

    fn create_dir(&self) -> anyhow::Result<()> {
        fs::DirBuilder::new()
            .create(&self.path)
            .context("Creating dir")
    }

    fn write_config(&self) -> anyhow::Result<()> {
        let mut file = fs::File::create(self.path.join("config.toml"))?;

        file.write_all(toml::to_string_pretty(&self.config)?.as_bytes())
            .context("Writing config")
    }

    fn write_remotes(&self) -> anyhow::Result<()> {
        let file = fs::File::create(self.path.join("remotes.json"))?;

        serde_json::to_writer_pretty(file, &self.remotes).context("Writing remotes")
    }

    fn write_locals(&self) -> anyhow::Result<()> {
        let locals_path = &self.path.join("locals");
        fs::DirBuilder::new()
            .recursive(true)
            .create(locals_path)
            .context("Creating locals dir")?;

        for name in self.locals.keys() {
            fs::File::create(locals_path.join(name))
                .with_context(|| format!("Creating `{}` in locals", name.to_string_lossy()))?;
        }

        Ok(())
    }

    fn save(&self) -> anyhow::Result<()> {
        self.create_dir()
            .and_then(|_| Self::write_config(self))
            .and_then(|_| Self::write_remotes(self))
            .and_then(|_| Self::write_locals(self))
            .context("Saving pool")
    }

    pub fn add_to_remotes(&mut self, the_mod: &SearchedMod) -> anyhow::Result<()> {
        self.remotes.insert(the_mod.slug().to_string());

        self.write_remotes()
    }
}
