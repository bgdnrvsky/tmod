use std::{
    cmp::Reverse,
    collections::{BTreeSet, HashMap},
    fmt::Display,
};

use anyhow::Context;
#[cfg(not(test))]
use loading::{Loading, Spinner};
use reqwest as rq;
use reqwest::blocking::Response;
use reqwest::Url;
use semver::VersionReq;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};

pub const TOKEN: &str = "$2a$10$bL4bIL5pUWqfcO7KQtnMReakwtfHbNKh6v1uTpKlzhwoueEJQnPnm"; // https://github.com/fn2006/PollyMC/wiki/CurseForge-Workaround

pub trait Fetchable
where
    Self: Sized,
{
    fn link() -> anyhow::Result<Url>;

    fn parse(response: Response) -> anyhow::Result<Self>;

    fn fetch(info: impl Display, params: Option<HashMap<String, String>>) -> anyhow::Result<Self> {
        #[cfg(not(test))]
        let loading = Self::loading_init(info);

        let mut url = Self::link()?;

        if let Some(params) = params {
            let mut parameters = url.query_pairs_mut();

            parameters.extend_pairs(params);
        }

        let response = Self::download(url)?;

        #[cfg(not(test))]
        Self::loading_end(loading);

        Self::parse(response)
    }

    #[cfg(not(test))]
    fn loading_init(info: impl Display) -> Loading {
        let loading = Loading::new(Spinner::default());
        loading.info(info);
        loading.text("Fetching");
        loading
    }

    fn download(url: Url) -> anyhow::Result<Response> {
        let mut req = rq::blocking::Request::new(rq::Method::GET, url);

        // Even if we don't need it
        let header_map = req.headers_mut();
        header_map.insert("x-api-key", rq::header::HeaderValue::from_static(TOKEN));

        let client = rq::blocking::Client::new();
        client.execute(req).context("Getting response from API")
    }

    #[cfg(not(test))]
    fn loading_end(loading: Loading) {
        loading.end();
    }
}

pub struct MinecraftId(usize);
pub struct MinecraftVersions(Vec<VersionReq>); // TODO: Use custom Version struct
pub struct ForgeVersions(HashMap<VersionReq, Vec<String>>); // TODO: Use custom Version struct
pub struct CurseForgeCategories(HashMap<String, usize>);

impl Fetchable for MinecraftId {
    fn link() -> anyhow::Result<Url> {
        Url::parse("https://api.curseforge.com/v1/games")
            .context("Url parsing for getting Minecraft id from CurseForge")
    }

    fn parse(response: Response) -> anyhow::Result<Self> {
        #[derive(Debug, Deserialize)]
        struct GamesList {
            data: Vec<GameEntry>,
        }

        #[derive(Debug, Deserialize)]
        struct GameEntry {
            id: usize,
            name: String,
            slug: String,
        }

        let games: GamesList = response.json()?;

        games
            .data
            .into_iter()
            .find(|entry| entry.name == "minecraft" || entry.slug == "minecraft")
            .map(|entry| entry.id)
            .map(Self)
            .context("Minecraft was not found in the list of games")
    }
}

impl Fetchable for MinecraftVersions {
    fn link() -> anyhow::Result<Url> {
        Url::parse("https://mc-versions-api.net/api/java")
            .context("Url parsing for getting Minecraft versions")
    }

    fn parse(response: Response) -> anyhow::Result<Self> {
        #[derive(Debug, Clone, Deserialize)]
        struct Data {
            result: Vec<VersionReq>,
        }

        response
            .json::<Data>()
            .context("Parsing Minecraft versions")
            .map(|v| v.result)
            .map(Self)
    }
}

impl Fetchable for ForgeVersions {
    fn link() -> anyhow::Result<Url> {
        Url::parse("https://mc-versions-api.net/api/forge")
            .context("Url parsing for getting forge versions")
    }

    fn parse(response: Response) -> anyhow::Result<Self> {
        #[derive(Debug, Clone, Deserialize)]
        struct Data {
            result: [HashMap<VersionReq, Vec<String>>; 1],
        }

        response
            .json::<Data>()
            .context("Deserializing Forge versions")
            .map(|Data { result: [version] }| version)
            .map(Self)
    }
}

impl Fetchable for CurseForgeCategories {
    fn link() -> anyhow::Result<Url> {
        let mut url = Url::parse("https://api.curseforge.com/v1/categories")
            .context("Url parsing for getting all categories")?;

        {
            let mut querys = url.query_pairs_mut();

            let id = MinecraftId::fetch("Getting Minecraft id from CurseForge", None)?.0;

            querys.append_pair("gameId", &format!("{id}"));
            querys.append_pair("classesOnly", "true");
        }

        Ok(url)
    }

    fn parse(response: Response) -> anyhow::Result<Self> {
        #[derive(Debug, Clone, Deserialize)]
        struct Data {
            data: Vec<CategoryEntry>,
        }

        #[derive(Debug, Clone, Deserialize)]
        struct CategoryEntry {
            name: String,
            id: usize,
        }

        let data = response.json::<Data>()?.data;

        Ok(Self(
            data.into_iter()
                .map(|entry| (entry.name, entry.id))
                .collect(),
        ))
    }
}

impl Fetchable for ModSearchList {
    fn link() -> anyhow::Result<Url> {
        let mut url = rq::Url::parse("https://api.curseforge.com/v1/mods/search").context("Parsing search mods url")?;

        {
            let mut querys = url.query_pairs_mut();

            let game_id = MinecraftId::fetch("Getting Minecraft id", None)?.0;
            let categories = CurseForgeCategories::fetch("Getting CurseForge categories", None)?.0;

            let class_id = categories.get("Mods").context("No category `Mods` found")?;

            querys.append_pair("gameId", &format!("{game_id}"));
            querys.append_pair("classId", &format!("{class_id}"));
        }

        Ok(url)
    }

    fn parse(response: Response) -> anyhow::Result<Self> {
        response.json().context("Deserializing searched mods")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModSearchList {
    #[serde(rename = "data")]
    mods: BTreeSet<Reverse<SearchedMod>>,
}

impl ModSearchList {
    pub fn mods(&self) -> &BTreeSet<Reverse<SearchedMod>> {
        &self.mods
    }
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct ModLinks {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "websiteUrl")]
    website: rq::Url,
    #[serde_as(as = "Option<DisplayFromStr>")]
    wiki: Option<rq::Url>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    issues: Option<rq::Url>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    source: Option<rq::Url>,
}

impl ModLinks {
    pub fn curseforge_url(&self) -> &rq::Url {
        &self.website
    }

    pub fn wiki_url(&self) -> Option<&rq::Url> {
        self.wiki.as_ref()
    }

    pub fn issues_url(&self) -> Option<&rq::Url> {
        self.issues.as_ref()
    }

    pub fn source_url(&self) -> Option<&rq::Url> {
        self.source.as_ref()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchedMod {
    id: usize,
    name: String,
    slug: String,
    links: ModLinks,
    #[serde(rename = "thumbsUpCount")]
    thumbs_up_count: usize,
    #[serde(rename = "downloadCount")]
    download_count: usize,
    #[serde(rename = "latestFiles")]
    files: Vec<ModFile>,
    #[serde(rename = "latestFilesIndexes")]
    indexes: Vec<ModFileIndex>,
}

impl SearchedMod {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn thumbs_up_count(&self) -> usize {
        self.thumbs_up_count
    }

    pub fn download_count(&self) -> usize {
        self.download_count
    }

    pub fn files(&self) -> &[ModFile] {
        &self.files
    }

    pub fn indexes(&self) -> &[ModFileIndex] {
        &self.indexes
    }

    pub fn links(&self) -> &ModLinks {
        &self.links
    }
}

impl PartialEq for SearchedMod {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for SearchedMod {}

impl PartialOrd for SearchedMod {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SearchedMod {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.download_count
            .cmp(&other.download_count)
            .then_with(|| self.thumbs_up_count.cmp(&other.thumbs_up_count))
    }
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

#[cfg(test)]
mod fetchers_test {
    use crate::fetchers::*;

    #[test]
    fn minecraft_id() {
        assert!(MinecraftId::fetch("", None).is_ok());
    }

    #[test]
    fn minecraft_versions() {
        assert!(MinecraftVersions::fetch("", None).is_ok_and(|versions| !versions.0.is_empty()));
    }

    #[test]
    fn forge_versions() {
        assert!(ForgeVersions::fetch("", None).is_ok_and(|map| map.0.keys().count() > 0));
    }
}
