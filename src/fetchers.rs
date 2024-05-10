use std::str::FromStr;

use anyhow::anyhow;
#[cfg(not(test))]
use loading::{Loading, Spinner};
use reqwest as rq;
use semver::VersionReq;
use serde::Deserialize;
use serde_json::{Map, Value};

pub const TOKEN: &str = "$2a$10$bL4bIL5pUWqfcO7KQtnMReakwtfHbNKh6v1uTpKlzhwoueEJQnPnm";
const GAMES_LIST_URL: &str = "https://api.curseforge.com/v1/games"; // https://github.com/fn2006/PollyMC/wiki/CurseForge-Workaround
const MINECRAFT_VERSIONS_LIST_URL: &str = "https://mc-versions-api.net/api/java";
const FORGE_VERSIONS_LIST_URL: &str = "https://mc-versions-api.net/api/forge";

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

    let minecraft_id = games
        .find_game("minecraft")
        .map(GameEntry::get_id)
        .ok_or(anyhow!("Minecraft was not found in the list of games"));

    #[cfg(not(test))]
    loading.end();

    minecraft_id
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

    let json: Value = serde_json::from_str(&response.text()?)?;

    #[cfg(not(test))]
    loading.end();

    json.get("result")
        .ok_or(anyhow!("No versions are present in response's JSON"))?
        .as_array()
        .ok_or(anyhow!("Result in JSON response was not an array"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or(anyhow!(
                    "One of the values in 'result' key of JSON was not a string"
                ))
                .and_then(|value| {
                    VersionReq::from_str(value).map_err(|e| anyhow!("Failed to parse version: {e}"))
                })
        })
        .collect()
}

pub fn get_forge_versions() -> anyhow::Result<Map<String, Value>> {
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

    let mut json: Value = serde_json::from_str(&response.text()?)?;

    #[cfg(not(test))]
    loading.end();

    json.get_mut("result")
        .ok_or(anyhow!("No versions are present in response's JSON"))?
        .as_array_mut()
        .ok_or(anyhow!("Result in JSON response was not an array"))?
        .get_mut(0)
        .ok_or(anyhow!("Array is expected to have at least one element"))?
        .as_object()
        .ok_or(anyhow!(
            "The only entry in resulting array is expected to be an object (map)"
        ))
        .cloned()
}

#[cfg(test)]
mod fetchers_test {
    use crate::fetchers::{get_minecraft_id, get_minecraft_versions};

    #[test]
    fn minecraft_id() {
        assert!(get_minecraft_id().is_ok());
    }

    #[test]
    fn minecraft_versions() {
        assert!(get_minecraft_versions().is_ok());
    }
}
