use std::{cell::OnceCell, collections::HashMap};

use anyhow::{anyhow, Context};
#[cfg(not(test))]
use loading::{Loading, Spinner};
use reqwest as rq;
use semver::VersionReq;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};

pub const TOKEN: &str = "$2a$10$bL4bIL5pUWqfcO7KQtnMReakwtfHbNKh6v1uTpKlzhwoueEJQnPnm";
const GAMES_LIST_URL: &str = "https://api.curseforge.com/v1/games"; // https://github.com/fn2006/PollyMC/wiki/CurseForge-Workaround
const MINECRAFT_VERSIONS_LIST_URL: &str = "https://mc-versions-api.net/api/java";
const FORGE_VERSIONS_LIST_URL: &str = "https://mc-versions-api.net/api/forge";
const SEARCH_MODS_URL: &str = "https://api.curseforge.com/v1/mods/search";

#[derive(Debug, Default)]
pub struct Fetcher {
    minecraft_id: OnceCell<anyhow::Result<usize>>,
    minecraft_versions: OnceCell<anyhow::Result<Vec<VersionReq>>>,
    forge_versions: OnceCell<anyhow::Result<HashMap<VersionReq, Vec<String>>>>,
}

impl Fetcher {
    pub fn get_minecraft_id(&self) -> Result<&usize, &anyhow::Error> {
        self.minecraft_id.get_or_init(fetch_minecraft_id).as_ref()
    }

    pub fn get_minecraft_versions(&self) -> Result<&Vec<VersionReq>, &anyhow::Error> {
        self.minecraft_versions
            .get_or_init(fetch_minecraft_versions)
            .as_ref()
    }

    pub fn get_forge_versions(&self) -> Result<&HashMap<VersionReq, Vec<String>>, &anyhow::Error> {
        self.forge_versions
            .get_or_init(fetch_forge_versions)
            .as_ref()
    }

    pub fn search_mods(&self, mod_slug: impl AsRef<str>) -> anyhow::Result<Vec<SearchedMod>> {
        let mut url = rq::Url::parse(SEARCH_MODS_URL).context("Parsing search mods url")?;

        {
            let mut querys = url.query_pairs_mut();

            let id = self
                .get_minecraft_id()
                .ok()
                .context("Getting Minecraft id")
                .copied()?;

            querys.append_pair("gameId", &format!("{id}"));
            querys.append_pair("searchFilter", mod_slug.as_ref());
        }

        let client = rq::blocking::Client::new();
        let mut req = rq::blocking::Request::new(rq::Method::GET, url);

        let header_map = req.headers_mut();
        header_map.insert("x-api-key", rq::header::HeaderValue::from_static(TOKEN));

        let response = client
            .execute(req)
            .context("Making call to CurseForge's API to search for mods")?;

        #[derive(Debug, Clone, Deserialize)]
        struct ModSearchList {
            #[serde(rename = "data")]
            mods: Vec<SearchedMod>,
        }

        Ok(response.json::<ModSearchList>()?.mods)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchedMod {
    id: usize,
    name: String,
    slug: String,
    #[serde(rename = "thumbsUpCount")]
    thumbs_up_count: usize,
    #[serde(rename = "downloadCount")]
    download_count: usize,
    #[serde(rename = "latestFiles")]
    files: Vec<ModFile>,
    #[serde(rename = "latestFilesIndexes")]
    indexes: Vec<ModFileIndex>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModFileIndex {
    #[serde(rename = "fileId")]
    id: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModDependency {
    #[serde(rename = "modId")]
    id: usize,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct ModFile {
    id: usize,
    #[serde(rename = "fileName")]
    file_name: String,
    #[serde(rename = "downloadCount")]
    download_count: usize,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "downloadUrl")]
    url: rq::Url,
    #[serde(rename = "gameVersions")]
    versions: Vec<String>,
    dependencies: Vec<ModDependency>,
}

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

pub fn fetch_minecraft_id() -> anyhow::Result<usize> {
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

pub fn fetch_minecraft_versions() -> anyhow::Result<Vec<VersionReq>> {
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

pub fn fetch_forge_versions() -> anyhow::Result<HashMap<VersionReq, Vec<String>>> {
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
        assert!(fetch_minecraft_id().is_ok());
    }

    #[test]
    fn minecraft_versions() {
        assert!(fetch_minecraft_versions().is_ok_and(|versions| versions.len() > 0));
    }

    #[test]
    fn forge_versions() {
        assert!(fetch_forge_versions().is_ok_and(|map| map.keys().count() > 0));
    }
}
