pub mod config;
pub mod loader;

use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Context;
use chrono::{DateTime, Utc};
use jars::{jar, JarOptionBuilder};
use zip::ZipWriter;

use crate::{
    fetcher::{
        mod_search::search_mod::{ModFile, SearchedMod},
        SEARCHER,
    },
    jar::JarMod,
};

use config::Config;

pub struct Pool {
    pub path: PathBuf,
    pub config: Config,
    /// Slug - File timestamp
    pub remotes: HashMap<String, DateTime<Utc>>,
    pub locals: Vec<JarMod>,
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

        // Check if `remotes.toml` file exists
        let remotes = {
            let file = find_entry("remotes.toml")?;

            anyhow::ensure!(
                file.metadata()?.is_file(),
                "`remotes.toml` is expected to be a file"
            );

            let content = fs::read_to_string(file.path()).context("Reading `remotes.toml`")?;
            toml::from_str(&content).context("Deserializing `remotes.toml`")?
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
                    entry.context("Reading entry").and_then(|entry| {
                        jar(entry.path(), JarOptionBuilder::default())
                            .context("Reading jar file")
                            .and_then(JarMod::try_from)
                    })
                })
                .collect::<Result<Vec<_>, _>>()?
        };

        Ok(Self {
            config,
            remotes,
            locals,
            path: dir_path.as_ref().to_owned(),
        })
    }

    fn create_dir(&self) -> anyhow::Result<()> {
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
        let mut file = File::create(self.remotes_path())?;
        let content = toml::to_string_pretty(&self.remotes).context("Serializing remotes")?;

        file.write_all(content.as_bytes())
            .context("Writing remotes")
    }

    fn write_locals(&self) -> anyhow::Result<()> {
        let locals_path = self.locals_path();

        fs::DirBuilder::new()
            .recursive(true)
            .create(&locals_path)
            .context("Creating locals dir")?;

        for jar_mod in self.locals.iter() {
            let path = locals_path.join(jar_mod.name()).with_extension("jar");
            let file = File::create(&path)
                .with_context(|| format!("Creating `{}` in locals", path.display()))?;

            let mut writer = ZipWriter::new(file);
            let options = zip::write::FileOptions::default();

            for (zip_filename, zip_file_content) in jar_mod.zip().files.iter() {
                // Mark the file where we are writing
                writer
                    .start_file(zip_filename, options)
                    .with_context(|| format!("Starting {} file in the jar", zip_filename))?;

                // Write the content
                writer
                    .write(zip_file_content)
                    .with_context(|| format!("Writing to {} in the jar", zip_filename))?;
            }

            writer
                .finish()
                .with_context(|| format!("Finishing writing to {} jar", path.display()))?;
        }

        Ok(())
    }

    pub fn save(&self) -> anyhow::Result<()> {
        self.create_dir()
            .and_then(|_| Self::write_config(self))
            .and_then(|_| Self::write_remotes(self))
            .and_then(|_| Self::write_locals(self))
            .context("Saving pool")
    }

    pub fn is_compatible(&self, the_mod: &SearchedMod) -> anyhow::Result<bool> {
        let file = SEARCHER
            .try_lock()
            .unwrap()
            .get_specific_mod_file(the_mod, &self.config, None);

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
                .try_lock()
                .unwrap()
                .search_mod_by_id(inc_id)
                .with_context(|| format!("Couldn't find the incompatibility id ({inc_id})"))?;

            for remote_slug in self.remotes.keys() {
                if inc.slug == *remote_slug {
                    return Ok(false);
                }
            }

            for local_slug in self.locals.iter().map(|jar| jar.name()) {
                if inc.slug == local_slug {
                    return Ok(false);
                }
            }
        }

        for local in self.locals.iter() {
            for incomp_slug in local.incompatibilities().keys() {
                if *incomp_slug == the_mod.slug {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    pub fn add_to_remotes_checked(
        &mut self,
        the_mod: &SearchedMod,
        mod_file: &ModFile,
    ) -> anyhow::Result<()> {
        if !self.is_compatible(the_mod)? {
            anyhow::bail!(
                "The mod {slug} is not compatible with the pool!",
                slug = the_mod.slug
            );
        }

        self.add_to_remotes_unchecked(the_mod, mod_file);
        Ok(())
    }

    pub fn add_to_remotes_unchecked(&mut self, the_mod: &SearchedMod, mod_file: &ModFile) {
        self.remotes.insert(the_mod.slug.to_string(), mod_file.date);
    }

    pub fn add_to_locals(&mut self, jar: JarMod) {
        self.locals.push(jar);
    }

    pub fn remove_mod(&mut self, name: &str) -> anyhow::Result<bool> {
        Ok(self.remove_from_locals(name)? || self.remove_from_remotes(name))
    }

    pub fn remove_from_locals(&mut self, name: &str) -> anyhow::Result<bool> {
        if let Some(idx) = self.locals.iter().position(|jar| jar.name() == name) {
            fs::remove_file(self.locals_path().join(name).with_extension("jar"))
                .with_context(|| format!("Deleting local JAR '{}.jar'", name))?;
            self.locals.swap_remove(idx);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn remove_from_remotes(&mut self, name: &str) -> bool {
        self.remotes.remove(name).is_some()
    }

    pub fn root_path(&self) -> &Path {
        &self.path
    }

    pub fn remotes_path(&self) -> PathBuf {
        self.root_path().join("remotes.toml")
    }

    pub fn locals_path(&self) -> PathBuf {
        self.root_path().join("locals/")
    }
}
