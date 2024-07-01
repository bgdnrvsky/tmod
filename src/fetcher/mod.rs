mod items;
mod mod_search;
pub mod searcher;

use std::{collections::HashMap, fmt::Display};

use loading::{Loading, Spinner};

use rq::Response;
use ureq as rq;
use url::Url;

use anyhow::Context;

pub const TOKEN: &str = "$2a$10$bL4bIL5pUWqfcO7KQtnMReakwtfHbNKh6v1uTpKlzhwoueEJQnPnm"; // https://github.com/fn2006/PollyMC/wiki/CurseForge-Workaround

/// Performs downloading, logging and parsing of some type from specified url with support for
/// custom runtime parameters for url
pub trait Fetchable
where
    Self: Sized,
{
    /// Url where GET will be performed
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
        rq::get(url.as_str())
            .set("x-api-key", TOKEN)
            .call()
            .context("Getting response from API")
    }

    fn loading_end(loading: Loading) {
        loading.end();
    }
}

#[derive(Debug, Clone, Default)]
struct AdditionalFetchParameters {
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

    pub fn add_query(&mut self, query: (impl AsRef<str>, impl AsRef<str>)) {
        self.queries
            .get_or_insert_with(HashMap::new)
            .insert(query.0.as_ref().to_string(), query.1.as_ref().to_string());
    }

    pub fn with_segments(self, segments: Vec<String>) -> Self {
        Self {
            queries: self.queries,
            path_segments: Some(segments),
        }
    }

    pub fn add_segment(&mut self, segment: String) {
        self.path_segments
            .get_or_insert_with(Vec::new)
            .push(segment);
    }
}

#[cfg(test)]
mod fetchers {
    use super::items::*;
    use super::AdditionalFetchParameters;
    use super::Fetchable;

    #[test]
    fn minecraft_id() -> anyhow::Result<()> {
        MinecraftId::fetch(AdditionalFetchParameters::default())?;

        Ok(())
    }

    #[test]
    fn minecraft_versions() -> anyhow::Result<()> {
        let versions = MinecraftVersions::fetch(AdditionalFetchParameters::default())?;
        assert!(!versions.is_empty());

        Ok(())
    }

    #[test]
    fn forge_versions() -> anyhow::Result<()> {
        let versions = ForgeVersions::fetch(AdditionalFetchParameters::default())?;
        assert!(!versions.is_empty());

        Ok(())
    }
}
