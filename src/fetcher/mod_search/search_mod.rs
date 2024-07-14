use std::fmt::Display;

use anyhow::Context;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};

use super::super::rq::*;
use super::super::{Fetchable, Url};
use super::mod_links::ModLinks;

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
    #[serde(rename = "latestFilesIndexes")]
    indexes: Vec<ModFileIndex>,
}

impl Fetchable for SearchedMod {
    fn info() -> impl Display {
        "Fetching Minecraft mod"
    }

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

    pub fn indexes(&self) -> &[ModFileIndex] {
        &self.indexes
    }

    pub fn links(&self) -> &ModLinks {
        &self.links
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }

    pub fn display(&self) -> display_builder::DisplayBuilder {
        display_builder::DisplayBuilder::new(self)
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
    #[allow(unused)]
    #[serde(rename = "fileId")]
    id: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModDependency {
    #[serde(rename = "modId")]
    #[allow(unused)]
    id: usize,
}

#[allow(unused)]
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

pub mod display_builder {
    use colored::Colorize;
    use std::fmt::Display;

    use crate::fetcher::mod_search::mod_links::display_builder::{
        DisplayBuilder as DisplayBuilderLinks, DisplayBuilderOptions as DisplayOptionsLinks,
    };

    use super::SearchedMod;

    /// Options to include while printing the searched mod
    #[derive(Debug, Clone, Copy)]
    pub struct DisplayBuilderOptions {
        with_id: bool,
        with_name: bool,
        with_slug: bool,
        with_summary: bool,
        with_links: bool,
        with_thumbs_up_count: bool,
        with_download_count: bool,
        with_files: bool,
        with_indexes: bool,
        links_options: Option<DisplayOptionsLinks>,
    }

    impl Default for DisplayBuilderOptions {
        fn default() -> Self {
            Self {
                with_id: true,
                with_name: true,
                with_slug: false,
                with_summary: true,
                with_links: false,
                with_thumbs_up_count: false,
                with_download_count: false,
                with_files: false,
                with_indexes: false,
                links_options: None,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct DisplayBuilder<'a> {
        the_mod: &'a SearchedMod,
        options: DisplayBuilderOptions,
    }

    impl<'a> DisplayBuilder<'a> {
        pub fn new(searched_mod: &'a SearchedMod) -> Self {
            Self {
                the_mod: searched_mod,
                options: DisplayBuilderOptions::default(),
            }
            .with_id(true)
            .with_name(true)
            .with_summary(true)
        }

        pub fn with_options(searched_mod: &'a SearchedMod, options: DisplayBuilderOptions) -> Self {
            Self {
                the_mod: searched_mod,
                options,
            }
        }

        pub fn with_id(mut self, value: bool) -> Self {
            self.options.with_id = value;
            self
        }

        pub fn with_name(mut self, value: bool) -> Self {
            self.options.with_name = value;
            self
        }

        pub fn with_slug(mut self, value: bool) -> Self {
            self.options.with_slug = value;
            self
        }

        pub fn with_summary(mut self, value: bool) -> Self {
            self.options.with_summary = value;
            self
        }

        pub fn with_links(mut self, value: bool) -> Self {
            self.options.with_links = value;
            self
        }

        pub fn with_thumbs_up_count(mut self, value: bool) -> Self {
            self.options.with_thumbs_up_count = value;
            self
        }

        pub fn with_download_count(mut self, value: bool) -> Self {
            self.options.with_download_count = value;
            self
        }

        pub fn with_files(mut self, value: bool) -> Self {
            self.options.with_files = value;
            self
        }

        pub fn with_indexes(mut self, value: bool) -> Self {
            self.options.with_indexes = value;
            self
        }

        pub fn with_links_builder(mut self, options: DisplayOptionsLinks) -> Self {
            self.options.links_options = Some(options);
            self
        }
    }

    impl Display for DisplayBuilder<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.options.with_id {
                write!(f, "(id: {}) ", self.the_mod.id().to_string().bold())?;
            }

            if self.options.with_name {
                write!(f, "{} ", self.the_mod.name().blue())?;
            }

            if self.options.with_slug {
                write!(f, "(slug: {}) ", self.the_mod.slug().bold())?;
            }

            if self.options.with_thumbs_up_count || self.options.with_download_count {
                write!(f, "[")?;
            }

            if self.options.with_thumbs_up_count {
                write!(f, "likes: {}", self.the_mod.thumbs_up_count)?;
            }

            if self.options.with_thumbs_up_count && self.options.with_download_count {
                write!(f, ", ")?;
            }

            if self.options.with_download_count {
                write!(f, "downloads: {}", self.the_mod.download_count)?;
            }

            if self.options.with_thumbs_up_count || self.options.with_download_count {
                write!(f, "] ")?;
            }

            if self.options.with_summary {
                write!(f, "- {}", self.the_mod.summary().italic())?;
            }

            if let Some(links_options) = self.options.links_options {
                let builder =
                    DisplayBuilderLinks::with_options(self.the_mod.links(), links_options);

                builder.fmt(f)?;
            }

            if self.options.with_files {
                writeln!(f, "Files:")?;

                write!(f, "{:?}", self.the_mod.files)?;
            }

            if self.options.with_indexes {
                writeln!(f, "Indexes:")?;

                write!(f, "{:?}", self.the_mod.indexes)?;
            }

            Ok(())
        }
    }
}
