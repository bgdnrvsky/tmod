use std::fmt::Display;

use anyhow::Context;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use ureq::Response;
use url::Url;

use super::super::Fetchable;
use super::mod_links::ModLinks;

pub mod display;

#[derive(Debug, Clone, Deserialize)]
pub struct SearchedMod {
    id: usize,
    name: String,
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

    pub fn display(&self) -> display::ModBuilder {
        display::ModBuilder::new(self)
    }

    pub fn display_with_options(&self, options: display::ModOptions) -> display::ModBuilder {
        display::ModBuilder::from_options(self, options)
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
