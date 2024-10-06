use std::{cmp::Reverse, collections::BTreeSet};

use serde::Deserialize;

use super::search_mod::SearchedMod;

use super::super::rq::*;

pub mod display;

#[derive(Debug, Clone, Deserialize)]
pub struct ModSearchList {
    #[serde(rename = "data")]
    pub mods: BTreeSet<Reverse<SearchedMod>>,
}

impl ModSearchList {
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
