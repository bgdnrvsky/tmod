use std::fmt::Formatter;
use std::{
    cmp::Reverse,
    collections::{BTreeSet, HashMap},
    fmt::Display,
};

use anyhow::Context;
use colored::Colorize;
use loading::{Loading, Spinner};
use reqwest as rq;
use rq::blocking::Response;
use rq::Url;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};

use crate::version::{ManyVersions, SingleVersion};

pub const TOKEN: &str = "$2a$10$bL4bIL5pUWqfcO7KQtnMReakwtfHbNKh6v1uTpKlzhwoueEJQnPnm"; // https://github.com/fn2006/PollyMC/wiki/CurseForge-Workaround

#[derive(Debug, Clone, Default)]
pub struct AdditionalFetchParameters {
    queries: Option<HashMap<String, String>>,
    path_segments: Option<Vec<String>>,
}

impl AdditionalFetchParameters {
    pub fn get_path_segments(&self) -> Option<&[String]> {
        self.path_segments.as_ref().map(AsRef::as_ref)
    }

    pub fn get_queries(&self) -> Option<&HashMap<String, String>> {
        self.queries.as_ref()
    }

    pub fn with_queries(self, queries: HashMap<String, String>) -> Self {
        Self {
            queries: Some(queries),
            path_segments: self.path_segments,
        }
    }

    pub fn with_segments(self, segments: Vec<String>) -> Self {
        Self {
            queries: self.queries,
            path_segments: Some(segments),
        }
    }
}

/// Performs downloading, logging and parsing of some type from specified url with support for
/// custom runtime parameters for url
pub trait Fetchable
where
    Self: Sized,
{
    /// Url where GET whill be performed
    fn link() -> anyhow::Result<Url>;

    /// from response's JSON to the datatype
    fn parse(response: Response) -> anyhow::Result<Self>;

    /// Performs the GET
    fn fetch(additional_parameters: AdditionalFetchParameters) -> anyhow::Result<Self> {
        #[cfg(not(test))]
        let loading = Self::loading_init();

        let mut url = Self::link()?;

        if let Some(queries) = additional_parameters.get_queries() {
            url.query_pairs_mut().extend_pairs(queries);
        }

        if let Some(segments) = additional_parameters.get_path_segments() {
            url.path_segments_mut()
                .ok()
                .context("Url cannot be a base")?
                .extend(segments);
        }

        let response = Self::download(url)?;

        #[cfg(not(test))]
        Self::loading_end(loading);

        Self::parse(response)
    }

    /// Message, displayed, by default: _i Fetching data_
    fn info() -> impl Display {
        "Fetching data"
    }

    /// Message, displayed by loading bar, by default: _% Fetching_
    fn description() -> impl Display {
        "Fetching"
    }

    fn loading_init() -> Loading {
        let loading = Loading::new(Spinner::default());
        loading.info(Self::info());
        loading.text(Self::description());
        loading
    }

    /// Performs plain GET
    fn download(url: Url) -> anyhow::Result<Response> {
        let mut req = rq::blocking::Request::new(rq::Method::GET, url);

        // Even if we don't need it
        let header_map = req.headers_mut();
        header_map.insert("x-api-key", rq::header::HeaderValue::from_static(TOKEN));

        let client = rq::blocking::Client::new();
        client.execute(req).context("Getting response from API")
    }

    fn loading_end(loading: Loading) {
        loading.end();
    }
}

/// Example JSON:
/// ```json
/// {
///   "data": [
///     {
///       "id": 0,
///       "name": "string",
///       "slug": "string",
///       "dateModified": "2019-08-24T14:15:22Z",
///       "assets": {
///         "iconUrl": "string",
///         "tileUrl": "string",
///         "coverUrl": "string"
///       },
///       "status": 1,
///       "apiStatus": 1
///     }
///   ],
///   "pagination": {
///     "index": 0,
///     "pageSize": 0,
///     "resultCount": 0,
///     "totalCount": 0
///   }
/// }
/// ```
pub struct MinecraftId(usize);
/// Example JSON:
/// ```json
/// {
///     "result":
///     [
///         "1.20.1",
///         "1.20",
///         "1.19.4",
///         "1.19.3",
///         "1.19.2",
///         "1.19.1",
///         "1.19",
///         "..."
///     ]
/// }
/// ```
pub struct MinecraftVersions(Vec<ManyVersions>);
/// Example JSON:
/// ```json
/// {
///     "result":["47.1.0", "47.0.50", "47.0.49", "47.0.46", "..."]
/// }
/// ```
pub struct ForgeVersions(HashMap<SingleVersion, Vec<SingleVersion>>);
/// Example JSON:
/// ```json
/// {
///   "data": [
///     {
///       "id": 0,
///       "gameId": 0,
///       "name": "string",
///       "slug": "string",
///       "url": "string",
///       "iconUrl": "string",
///       "dateModified": "2019-08-24T14:15:22Z",
///       "isClass": true,
///       "classId": 0,
///       "parentCategoryId": 0,
///       "displayIndex": 0
///     }
///   ]
/// }
/// ```
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

    fn info() -> impl Display {
        "Getting Minecraft id from CurseForge"
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
            result: Vec<ManyVersions>,
        }

        response
            .json::<Data>()
            .context("Parsing Minecraft versions")
            .map(|v| v.result)
            .map(Self)
    }

    fn info() -> impl Display {
        "Getting Minecraft versions"
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
            result: [HashMap<SingleVersion, Vec<SingleVersion>>; 1],
        }

        response
            .json::<Data>()
            .context("Deserializing Forge versions")
            .map(|Data { result: [version] }| version)
            .map(Self)
    }

    fn info() -> impl Display {
        "Getting Forge versions from CurseForge"
    }
}

impl Fetchable for CurseForgeCategories {
    fn link() -> anyhow::Result<Url> {
        let mut url = Url::parse("https://api.curseforge.com/v1/categories")
            .context("Url parsing for getting all categories")?;

        {
            let mut queries = url.query_pairs_mut();

            let id = MinecraftId::fetch(AdditionalFetchParameters::default())?.0;

            queries.append_pair("gameId", &format!("{id}"));
            queries.append_pair("classesOnly", "true");
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

    fn info() -> impl Display {
        "Getting game categories from CurseForge"
    }
}

impl Fetchable for ModSearchList {
    fn link() -> anyhow::Result<Url> {
        let mut url = Url::parse("https://api.curseforge.com/v1/mods/search")
            .context("Parsing search mods url")?;

        {
            let mut queries = url.query_pairs_mut();

            let game_id = MinecraftId::fetch(AdditionalFetchParameters::default())?.0;
            let categories = CurseForgeCategories::fetch(AdditionalFetchParameters::default())?.0;

            let class_id = categories.get("Mods").context("No category `Mods` found")?;

            queries.append_pair("gameId", &format!("{game_id}"));
            queries.append_pair("classId", &format!("{class_id}"));
        }

        Ok(url)
    }

    fn parse(response: Response) -> anyhow::Result<Self> {
        response.json().context("Deserializing searched mods")
    }

    fn info() -> impl Display {
        "Searching for mod"
    }
}

impl Fetchable for SearchedMod {
    fn link() -> anyhow::Result<Url> {
        Url::parse("https://api.curseforge.com/v1/mods")
            .context("Parsing Url for fetching mod by it's id")
    }

    fn parse(response: Response) -> anyhow::Result<Self> {
        #[derive(Debug, Clone, Deserialize)]
        struct Data {
            data: SearchedMod,
        }

        response
            .json::<Data>()
            .context("Deserializing response")
            .map(|data| data.data)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModSearchList {
    #[serde(rename = "data")]
    mods: BTreeSet<Reverse<SearchedMod>>,
}

impl Display for ModSearchList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Found {n} mods:", n = self.mods.len())?;

        for Reverse(m) in &self.mods {
            writeln!(f, "- {m}")?;
        }

        Ok(())
    }
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
    website: Url,
    #[serde_as(as = "Option<DisplayFromStr>")]
    wiki: Option<Url>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    issues: Option<Url>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    source: Option<Url>,
}

impl ModLinks {
    pub fn curseforge_url(&self) -> &Url {
        &self.website
    }

    pub fn wiki_url(&self) -> Option<&Url> {
        self.wiki.as_ref()
    }

    pub fn issues_url(&self) -> Option<&Url> {
        self.issues.as_ref()
    }

    pub fn source_url(&self) -> Option<&Url> {
        self.source.as_ref()
    }
}

#[derive(Debug, Clone, Deserialize)]
/// Example JSON:
/// ```json
/// {
///   "data": {
///     "id": 0,
///     "gameId": 0,
///     "name": "string",
///     "slug": "string",
///     "links": {
///       "websiteUrl": "string",
///       "wikiUrl": "string",
///       "issuesUrl": "string",
///       "sourceUrl": "string"
///     },
///     "summary": "string",
///     "status": 1,
///     "downloadCount": 0,
///     "isFeatured": true,
///     "primaryCategoryId": 0,
///     "categories": [
///       {
///         "id": 0,
///         "gameId": 0,
///         "name": "string",
///         "slug": "string",
///         "url": "string",
///         "iconUrl": "string",
///         "dateModified": "2019-08-24T14:15:22Z",
///         "isClass": true,
///         "classId": 0,
///         "parentCategoryId": 0,
///         "displayIndex": 0
///       }
///     ],
///     "classId": 0,
///     "authors": [
///       {
///         "id": 0,
///         "name": "string",
///         "url": "string"
///       }
///     ],
///     "logo": {
///       "id": 0,
///       "modId": 0,
///       "title": "string",
///       "description": "string",
///       "thumbnailUrl": "string",
///       "url": "string"
///     },
///     "screenshots": [
///       {
///         "id": 0,
///         "modId": 0,
///         "title": "string",
///         "description": "string",
///         "thumbnailUrl": "string",
///         "url": "string"
///       }
///     ],
///     "mainFileId": 0,
///     "latestFiles": [
///       {
///         "id": 0,
///         "gameId": 0,
///         "modId": 0,
///         "isAvailable": true,
///         "displayName": "string",
///         "fileName": "string",
///         "releaseType": 1,
///         "fileStatus": 1,
///         "hashes": [
///           {
///             "value": "string",
///             "algo": 1
///           }
///         ],
///         "fileDate": "2019-08-24T14:15:22Z",
///         "fileLength": 0,
///         "downloadCount": 0,
///         "fileSizeOnDisk": 0,
///         "downloadUrl": "string",
///         "gameVersions": [
///           "string"
///         ],
///         "sortableGameVersions": [
///           {
///             "gameVersionName": "string",
///             "gameVersionPadded": "string",
///             "gameVersion": "string",
///             "gameVersionReleaseDate": "2019-08-24T14:15:22Z",
///             "gameVersionTypeId": 0
///           }
///         ],
///         "dependencies": [
///           {
///             "modId": 0,
///             "relationType": 1
///           }
///         ],
///         "exposeAsAlternative": true,
///         "parentProjectFileId": 0,
///         "alternateFileId": 0,
///         "isServerPack": true,
///         "serverPackFileId": 0,
///         "isEarlyAccessContent": true,
///         "earlyAccessEndDate": "2019-08-24T14:15:22Z",
///         "fileFingerprint": 0,
///         "modules": [
///           {
///             "name": "string",
///             "fingerprint": 0
///           }
///         ]
///       }
///     ],
///     "latestFilesIndexes": [
///       {
///         "gameVersion": "string",
///         "fileId": 0,
///         "filename": "string",
///         "releaseType": 1,
///         "gameVersionTypeId": 0,
///         "modLoader": 0
///       }
///     ],
///     "latestEarlyAccessFilesIndexes": [
///       {
///         "gameVersion": "string",
///         "fileId": 0,
///         "filename": "string",
///         "releaseType": 1,
///         "gameVersionTypeId": 0,
///         "modLoader": 0
///       }
///     ],
///     "dateCreated": "2019-08-24T14:15:22Z",
///     "dateModified": "2019-08-24T14:15:22Z",
///     "dateReleased": "2019-08-24T14:15:22Z",
///     "allowModDistribution": true,
///     "gamePopularityRank": 0,
///     "isAvailable": true,
///     "thumbsUpCount": 0,
///     "rating": 0
///   }
/// }
/// ```
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

impl Display for SearchedMod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(id: {mod_id}) {mod_name} - {website_url}",
            mod_id = format!("{}", self.id).bold(),
            mod_name = self.name.blue(),
            website_url = format!("{}", self.links.website).italic()
        )
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
    url: Url,
    #[serde(rename = "gameVersions")]
    versions: Vec<String>,
    dependencies: Vec<ModDependency>,
}

#[cfg(test)]
mod fetchers_test {
    use crate::fetchers::*;

    #[test]
    fn minecraft_id() {
        assert!(MinecraftId::fetch(AdditionalFetchParameters::default()).is_ok());
    }

    #[test]
    fn minecraft_versions() {
        assert!(
            MinecraftVersions::fetch(AdditionalFetchParameters::default())
                .is_ok_and(|versions| !versions.0.is_empty())
        );
    }

    #[test]
    fn forge_versions() {
        assert!(
            ForgeVersions::fetch(AdditionalFetchParameters::default()).is_ok_and(|map| map
                .0
                .keys()
                .count()
                > 0)
        );
    }
}
