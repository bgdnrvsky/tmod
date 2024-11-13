use std::{
    fs::{self, File},
    io::Write,
};

use anyhow::Context;

use super::Pool;

impl Pool {
    pub fn save(&self) -> anyhow::Result<()> {
        self.create_pool_dir()
            .and_then(|_| Self::write_config(self))
            .and_then(|_| Self::write_remotes(self))
            .context("Saving pool")
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
}
