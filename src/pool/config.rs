use std::{fs, path::Path};

use anyhow::Context;
use dialoguer::Input;
use serde::{Deserialize, Serialize};

use super::loader::Loader;

/// Basic configuration including: loader (forge or fabric) and the goal game version
///
/// Example configuration:
/// ```toml
/// game_version = "1.20.1"
///
/// [loader]
/// kind = "forge"
/// version = "47.2.0"
/// ```
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Config {
    /// Minecraft mod loader (forge or fabric)
    loader: Loader,
    /// Minecraft version target
    game_version: String,
}

impl Config {
    pub fn init() -> anyhow::Result<Self> {
        let loader = Loader::prompt()?;
        let game_version = Input::<String>::new()
            .with_prompt("Game version")
            .interact()
            .unwrap()
            .parse()?;

        Ok(Self {
            loader,
            game_version,
        })
    }

    pub fn from_toml<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path).context("Reading file content")?;

        toml::from_str(&content).context("Deserializing")
    }

    pub fn loader(&self) -> &Loader {
        &self.loader
    }

    pub fn game_version(&self) -> &str {
        &self.game_version
    }
}

#[cfg(test)]
mod config_deserializer_tests {
    use super::Config;

    #[test]
    fn valid() {
        let config = toml::from_str::<Config>(
            r#"
            game_version = "1.20.1"

            [loader]
            kind = "forge"
            version = "47.2.0"
            "#,
        );

        assert!(config.is_ok());
    }

    #[test]
    #[should_panic]
    fn missing() {
        toml::from_str::<Config>(
            r#"
            [loader]
            kind = "forge"
            version = "47.2.0"
            "#,
        )
        .expect("Should fail since game version is missing");
    }
}
