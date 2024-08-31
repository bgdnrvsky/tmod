use std::{cmp::Reverse, collections::BTreeSet, fmt::Display};

use anyhow::Context;
use serde::Deserialize;

use super::super::Fetchable;
use super::search_mod::SearchedMod;

use super::super::{rq::*, Url};

pub mod display;

#[derive(Debug, Clone, Deserialize)]
pub struct ModSearchList {
    #[serde(rename = "data")]
    mods: BTreeSet<Reverse<SearchedMod>>,
}

impl Fetchable for ModSearchList {
    fn link() -> Url {
        Url::parse("https://api.curseforge.com/v1/mods/search").unwrap()
    }

    fn parse(response: Response) -> anyhow::Result<Self> {
        response.into_json().context("Deserializing searched mods")
    }

    fn info() -> impl Display {
        "Searching for mod"
    }
}

impl ModSearchList {
    pub fn mods(&self) -> &BTreeSet<Reverse<SearchedMod>> {
        &self.mods
    }

    pub fn count(&self) -> usize {
        self.mods.len()
    }

    pub fn display(&self) -> display::ListBuilder {
        display::ListBuilder::new(self)
    }

    pub fn to_single_mod(mut self) -> Result<SearchedMod, usize> {
        let count = self.count();
        if count != 1 {
            return Err(count);
        }

        Ok(self.mods.pop_first().unwrap().0)
    }
}
