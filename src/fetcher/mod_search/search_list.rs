use std::{cmp::Reverse, collections::BTreeSet, fmt::Display};

use anyhow::Context;
use serde::Deserialize;

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

impl ModSearchList {
    pub fn mods(&self) -> &BTreeSet<Reverse<SearchedMod>> {
        &self.mods
    }

    pub fn count(&self) -> usize {
        self.mods.len()
    }

    pub fn display(&self) -> display_builder::ModSearchListDisplayBuilder {
        display_builder::ModSearchListDisplayBuilder::new(self)
    }
}

pub mod display_builder {
    use std::{cmp::Reverse, fmt::Display};

    use crate::fetcher::mod_search::search_mod::display_builder::{
        DisplayOptions as SearchedModDisplayOptions, SearchedModDisplayBuilder,
    };

    use super::ModSearchList;

    /// Options to include while printing the searched mod
    #[derive(Debug, Clone, Copy, Default)]
    pub struct DisplayOptions {
        with_count: bool,
        searched_mod_options: Option<SearchedModDisplayOptions>,
    }

    #[derive(Debug, Clone)]
    pub struct ModSearchListDisplayBuilder<'a> {
        the_list: &'a ModSearchList,
        options: DisplayOptions,
    }

    impl<'a> ModSearchListDisplayBuilder<'a> {
        pub fn new(list: &'a ModSearchList) -> Self {
            Self {
                the_list: list,
                options: DisplayOptions::default(),
            }
            .with_count(true)
        }

        pub fn with_options(list: &'a ModSearchList, options: DisplayOptions) -> Self {
            Self {
                the_list: list,
                options,
            }
        }

        pub fn with_count(mut self, value: bool) -> Self {
            self.options.with_count = value;
            self
        }

        pub fn with_searched_mod_builder(mut self, builder: SearchedModDisplayOptions) -> Self {
            self.options.searched_mod_options = Some(builder);
            self
        }
    }

    impl Display for ModSearchListDisplayBuilder<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.options.with_count {
                if self.the_list.count() == 0 {
                    return write!(f, "No mods found!");
                }

                writeln!(f, "Found {n} mod(s)", n = self.the_list.count())?;
            }

            for Reverse(mmod) in &self.the_list.mods {
                let search_mod_options = self.options.searched_mod_options.unwrap_or_default();

                writeln!(
                    f,
                    "- {}",
                    SearchedModDisplayBuilder::with_options(mmod, search_mod_options)
                )?;
            }

            Ok(())
        }
    }
}
