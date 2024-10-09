use std::{fs, path::Path};

use anyhow::Context;
use dialoguer::Input;
use serde::{Deserialize, Serialize};

use super::loader::Loaders;

/// Basic configuration including: loader (forge or fabric) and the goal game version
///
/// Example configuration:
/// ```toml
/// game_version = "1.20.1"
/// loader = "forge"
/// ```
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Config {
    /// Minecraft mod loader
    pub loader: Loaders,
    /// Minecraft version target
    pub game_version: String,
}

impl Config {
    pub fn init() -> anyhow::Result<Self> {
        let loader = Loaders::prompt()?;
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
}
