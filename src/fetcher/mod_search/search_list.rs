use std::{
    cmp::Reverse,
    collections::BTreeSet,
    fmt::{Display, Formatter},
};

use serde::Deserialize;
use anyhow::Context;

use super::super::Fetchable;
use super::search_mod::SearchedMod;

use super::super::{rq::*, Url};

#[derive(Debug, Clone, Deserialize)]
pub struct ModSearchList {
    #[serde(rename = "data")]
    mods: BTreeSet<Reverse<SearchedMod>>,
}

impl Fetchable for ModSearchList {
    fn link() -> anyhow::Result<Url> {
        Url::parse("https://api.curseforge.com/v1/mods/search").context("Parsing search mods url")
    }

    fn parse(response: Response) -> anyhow::Result<Self> {
        response.into_json().context("Deserializing searched mods")
    }

    fn info() -> impl Display {
        "Searching for mod"
    }
}

impl Display for ModSearchList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Found {n} mod(s):", n = self.mods.len())?;

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

    pub fn count(&self) -> usize {
        self.mods.len()
    }
}
