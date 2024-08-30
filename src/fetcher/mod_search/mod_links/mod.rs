use super::super::Url;

use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};

pub mod display;

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
    pub fn website(&self) -> &Url {
        &self.website
    }

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

    pub fn display(&self) -> display::DisplayBuilder {
        display::DisplayBuilder::new(self)
    }
}
