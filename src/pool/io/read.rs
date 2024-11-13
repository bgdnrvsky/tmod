use std::{
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use anyhow::Context;

use super::super::Config;
use super::Pool;

impl Pool {
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
}
