use clap::Args;
use colored::Colorize;
use std::fmt::Display;
use url::Url;

use super::ModLinks;

/// Options to include while printing the searched mod
#[derive(Debug, Clone, Copy, Default, Args)]
pub struct LinksOptions {
    #[arg(long, default_value_t = false)]
    pub with_website: bool,
    #[arg(long, default_value_t = false)]
    pub with_wiki: bool,
    #[arg(long, default_value_t = false)]
    pub with_issues: bool,
    #[arg(long, default_value_t = false)]
    pub with_source: bool,
}

#[derive(Debug, Clone)]
pub struct LinksBuilder<'a> {
    the_links: &'a ModLinks,
    options: LinksOptions,
}

impl<'a> LinksBuilder<'a> {
    pub fn new(links: &'a ModLinks) -> Self {
        Self {
            the_links: links,
            options: LinksOptions::default(),
        }
        .with_source(true)
        .with_website(true)
        .with_wiki(true)
        .with_issues(true)
    }

    pub fn with_options(links: &'a ModLinks, options: LinksOptions) -> Self {
        Self {
            the_links: links,
            options,
        }
    }

    pub fn with_website(mut self, value: bool) -> Self {
        self.options.with_website = value;
        self
    }

    pub fn with_wiki(mut self, value: bool) -> Self {
        self.options.with_wiki = value;
        self
    }

    pub fn with_issues(mut self, value: bool) -> Self {
        self.options.with_issues = value;
        self
    }

    pub fn with_source(mut self, value: bool) -> Self {
        self.options.with_source = value;
        self
    }
}

impl Display for LinksBuilder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let the = self.the_links;
        let formatted = |url: &Option<Url>| {
            url.as_ref()
                .map(Url::as_str)
                .unwrap_or_else(|| "No source!")
                .italic()
        };

        writeln!(f, "\nLinks:")?;
        if self.options.with_website {
            writeln!(f, "Website: {}", the.website.as_str().italic())?;
        }
        if self.options.with_wiki {
            writeln!(f, "Wiki: {}", formatted(&the.wiki))?;
        }
        if self.options.with_issues {
            writeln!(f, "Issues: {}", formatted(&the.issues))?;
        }
        if self.options.with_source {
            writeln!(f, "Source: {}", formatted(&the.source))?;
        }

        Ok(())
    }
}
