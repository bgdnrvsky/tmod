use clap::Args;
use colored::Colorize;
use std::fmt::Display;

use crate::fetcher::mod_search::mod_links::display::{LinksBuilder, LinksOptions};

use super::SearchedMod;

/// Options to include while printing the searched mod
#[derive(Debug, Clone, Copy, Args)]
pub struct ModOptions {
    #[arg(long, default_value_t = true)]
    pub with_id: bool,
    #[arg(long, default_value_t = true)]
    pub with_name: bool,
    /// Include the mod identifier name (might be different from the mod name)
    #[arg(long, default_value_t = false)]
    pub with_slug: bool,
    #[arg(long, default_value_t = true)]
    pub with_summary: bool,
    #[arg(long, default_value_t = false)]
    pub with_thumbs_up_count: bool,
    #[arg(long, default_value_t = false)]
    pub with_download_count: bool,
    #[clap(flatten)]
    pub links_options: Option<LinksOptions>,
}

impl Default for ModOptions {
    fn default() -> Self {
        Self {
            with_id: true,
            with_name: true,
            with_slug: false,
            with_summary: true,
            with_thumbs_up_count: false,
            with_download_count: false,
            links_options: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModBuilder<'a> {
    the_mod: &'a SearchedMod,
    options: ModOptions,
}

impl<'a> ModBuilder<'a> {
    pub fn new(searched_mod: &'a SearchedMod) -> Self {
        Self {
            the_mod: searched_mod,
            options: ModOptions::default(),
        }
        .with_id(true)
        .with_name(true)
        .with_summary(true)
    }

    pub fn from_options(searched_mod: &'a SearchedMod, options: ModOptions) -> Self {
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

    pub fn with_thumbs_up_count(mut self, value: bool) -> Self {
        self.options.with_thumbs_up_count = value;
        self
    }

    pub fn with_download_count(mut self, value: bool) -> Self {
        self.options.with_download_count = value;
        self
    }

    pub fn with_links_builder(mut self, options: LinksOptions) -> Self {
        self.options.links_options = Some(options);
        self
    }
}

impl Display for ModBuilder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.options.with_id {
            write!(f, "(id: {}) ", self.the_mod.id.to_string().bold())?;
        }

        if self.options.with_name {
            write!(f, "{} ", self.the_mod.name.blue())?;
        }

        if self.options.with_slug {
            write!(f, "(slug: {}) ", self.the_mod.slug.bold())?;
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
            write!(f, "- {}", self.the_mod.summary.italic())?;
        }

        if let Some(links_options) = self.options.links_options {
            let builder = LinksBuilder::with_options(&self.the_mod.links, links_options);

            builder.fmt(f)?;
        }

        Ok(())
    }
}
