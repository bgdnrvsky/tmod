use std::{cmp::Reverse, fmt::Display};

use crate::fetcher::mod_search::search_mod::display::{
    ModBuilder as DisplayBuilderMod, ModOptions as DisplayOptionsMod,
};

use super::ModSearchList;

/// Options to include while printing the searched mod
#[derive(Debug, Clone, Copy, Default)]
pub struct ListOptions {
    with_count: bool,
    searched_mod_options: Option<DisplayOptionsMod>,
}

#[derive(Debug, Clone)]
pub struct ListBuilder<'a> {
    the_list: &'a ModSearchList,
    options: ListOptions,
}

impl<'a> ListBuilder<'a> {
    pub fn new(list: &'a ModSearchList) -> Self {
        Self {
            the_list: list,
            options: ListOptions::default(),
        }
        .with_count(true)
    }

    pub fn with_options(list: &'a ModSearchList, options: ListOptions) -> Self {
        Self {
            the_list: list,
            options,
        }
    }

    pub fn with_count(mut self, value: bool) -> Self {
        self.options.with_count = value;
        self
    }

    pub fn with_searched_mod_builder(mut self, builder: DisplayOptionsMod) -> Self {
        self.options.searched_mod_options = Some(builder);
        self
    }
}

impl Display for ListBuilder<'_> {
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
                DisplayBuilderMod::from_options(mmod, search_mod_options)
            )?;
        }

        Ok(())
    }
}
