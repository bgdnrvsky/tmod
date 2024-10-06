use super::super::Url;

use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};

pub mod display;

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct ModLinks {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "websiteUrl")]
    pub website: Url,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub wiki: Option<Url>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub issues: Option<Url>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub source: Option<Url>,
}

impl ModLinks {
    pub fn display(&self) -> display::LinksBuilder {
        display::LinksBuilder::new(self)
    }
}
