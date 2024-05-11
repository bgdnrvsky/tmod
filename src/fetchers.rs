use std::collections::HashMap;

use anyhow::{anyhow, Context};
#[cfg(not(test))]
use loading::{Loading, Spinner};
use reqwest as rq;
use semver::VersionReq;
use serde::Deserialize;

pub const TOKEN: &str = "$2a$10$bL4bIL5pUWqfcO7KQtnMReakwtfHbNKh6v1uTpKlzhwoueEJQnPnm";
const GAMES_LIST_URL: &str = "https://api.curseforge.com/v1/games"; // https://github.com/fn2006/PollyMC/wiki/CurseForge-Workaround
const MINECRAFT_VERSIONS_LIST_URL: &str = "https://mc-versions-api.net/api/java";
const FORGE_VERSIONS_LIST_URL: &str = "https://mc-versions-api.net/api/forge";

#[derive(Debug, Clone, Deserialize)]
struct ForgeVersions {
    pub result: [HashMap<VersionReq, Vec<String>>; 1], // TODO: Use custom Version struct
}

#[derive(Debug, Clone, Deserialize)]
struct MinecraftVersions {
    pub result: Vec<VersionReq>, // TODO: Use custom Version struct
}

#[derive(Debug, Deserialize)]
struct GamesList {
    data: Vec<GameEntry>,
}

impl GamesList {
    fn find_game(&self, game_name: impl AsRef<str>) -> Option<&GameEntry> {
        self.data.iter().find(|entry| {
            entry.get_name() == game_name.as_ref() || entry.get_slug() == game_name.as_ref()
        })
    }
}

#[derive(Debug, Deserialize)]
struct GameEntry {
    id: usize,
    name: String,
    slug: String,
}

impl GameEntry {
    fn get_slug(&self) -> &str {
        &self.name
    }

    fn get_name(&self) -> &str {
        &self.slug
    }

    fn get_id(&self) -> usize {
        self.id
    }
}

pub fn get_minecraft_id() -> anyhow::Result<usize> {
    #[cfg(not(test))]
    let loading = Loading::new(Spinner::default());

    #[cfg(not(test))]
    loading.info(format!(
        "Retrieving Minecraft's ID from {url}",
        url = GAMES_LIST_URL
    ));

    #[cfg(not(test))]
    loading.text("Decoding game entries");

    let mut req = rq::blocking::Request::new(rq::Method::GET, rq::Url::parse(GAMES_LIST_URL)?);

    let header_map = req.headers_mut();
    header_map.insert("x-api-key", rq::header::HeaderValue::from_static(TOKEN));

    let client = rq::blocking::Client::new();
    let response = client.execute(req)?;

    let games: GamesList = response.json()?;

    #[cfg(not(test))]
    loading.end();

    games
        .find_game("minecraft")
        .map(GameEntry::get_id)
        .context("Minecraft was not found in the list of games")
}

pub fn get_minecraft_versions() -> anyhow::Result<Vec<VersionReq>> {
    #[cfg(not(test))]
    let loading = Loading::new(Spinner::default());

    #[cfg(not(test))]
    loading.info(format!(
        "Retrieving Minecraft's versions from {url}",
        url = MINECRAFT_VERSIONS_LIST_URL
    ));

    #[cfg(not(test))]
    loading.text("Downloading");

    let req = rq::blocking::Request::new(
        rq::Method::GET,
        rq::Url::parse(MINECRAFT_VERSIONS_LIST_URL)?,
    );

    let client = rq::blocking::Client::new();
    let response = client.execute(req)?;

    #[cfg(not(test))]
    loading.end();

    serde_json::from_str::<MinecraftVersions>(&response.text()?)
        .with_context(|| anyhow!("Failed to deserialize minecraft versions"))
        .map(|v| v.result)
}

pub fn get_forge_versions() -> anyhow::Result<HashMap<VersionReq, Vec<String>>> {
    #[cfg(not(test))]
    let loading = Loading::new(Spinner::default());

    #[cfg(not(test))]
    loading.info(format!(
        "Retrieving Forge's versions from {url}",
        url = FORGE_VERSIONS_LIST_URL
    ));

    #[cfg(not(test))]
    loading.text("Downloading");

    let req = rq::blocking::Request::new(rq::Method::GET, rq::Url::parse(FORGE_VERSIONS_LIST_URL)?);

    let client = rq::blocking::Client::new();
    let response = client.execute(req)?;

    #[cfg(not(test))]
    loading.end();

    serde_json::from_str(&response.text()?)
        .with_context(|| anyhow!("Failed to deserialize forge versions"))
        .map(|versions: ForgeVersions| versions.result.first().unwrap().clone())
}

#[cfg(test)]
mod fetchers_test {
    use crate::fetchers::*;

    #[test]
    fn minecraft_id() {
        assert!(get_minecraft_id().is_ok());
    }

    #[test]
    fn minecraft_versions() {
        assert!(get_minecraft_versions().is_ok_and(|versions| versions.len() > 0));
    }

    #[test]
    fn forge_versions() {
        assert!(get_forge_versions().is_ok_and(|map| map.keys().count() > 0));
    }
}
