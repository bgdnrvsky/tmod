use std::fmt::Display;

use anyhow::Context;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use ureq::Response;
use url::Url;

use super::super::Fetchable;
use super::mod_links::ModLinks;

pub mod display;

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
#[derive(Debug, Clone, Deserialize)]
pub struct SearchedMod {
    id: usize,
    name: String,
    #[allow(unused)]
    slug: String,
    summary: String,
    links: ModLinks,
    #[serde(rename = "thumbsUpCount")]
    thumbs_up_count: usize,
    #[serde(rename = "downloadCount")]
    download_count: usize,
    #[serde(rename = "latestFiles")]
    files: Vec<ModFile>,
}

impl Fetchable for SearchedMod {
    fn info() -> impl Display {
        "Fetching Minecraft mod"
    }

    fn link() -> Url {
        Url::parse("https://api.curseforge.com/v1/mods").unwrap()
    }

    fn parse(response: Response) -> anyhow::Result<Self> {
        #[derive(Debug, Clone, Deserialize)]
        struct Data {
            data: SearchedMod,
        }

        response
            .into_json::<Data>()
            .context("Deserializing response")
            .map(|data| data.data)
    }
}

impl SearchedMod {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn slug(&self) -> &str {
        &self.slug
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

    pub fn links(&self) -> &ModLinks {
        &self.links
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }

    pub fn display(&self) -> display::Builder {
        display::Builder::new(self)
    }

    pub fn display_with_options(&self, options: display::Options) -> display::Builder {
        display::Builder::from_options(self, options)
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
pub struct ModDependency {
    #[serde(rename = "modId")]
    #[allow(unused)]
    id: usize,
}

impl ModDependency {
    pub fn id(&self) -> usize {
        self.id
    }
}

#[allow(unused)]
#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct ModFile {
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

impl ModFile {
    pub fn versions(&self) -> &[String] {
        &self.versions
    }

    pub fn dependencies(&self) -> &[ModDependency] {
        &self.dependencies
    }
}
