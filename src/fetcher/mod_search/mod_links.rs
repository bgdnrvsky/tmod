use super::super::Url;

use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};

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

    pub fn display(&self) -> display_builder::DisplayBuilder {
        display_builder::DisplayBuilder::new(self)
    }
}

pub mod display_builder {
    use colored::Colorize;
    use std::fmt::Display;
    use url::Url;

    use super::ModLinks;

    /// Options to include while printing the searched mod
    #[derive(Debug, Clone, Copy, Default)]
    pub struct DisplayBuilderOptions {
        pub with_website: bool,
        pub with_wiki: bool,
        pub with_issues: bool,
        pub with_source: bool,
    }

    #[derive(Debug, Clone)]
    pub struct DisplayBuilder<'a> {
        the_links: &'a ModLinks,
        options: DisplayBuilderOptions,
    }

    impl<'a> DisplayBuilder<'a> {
        pub fn new(links: &'a ModLinks) -> Self {
            Self {
                the_links: links,
                options: DisplayBuilderOptions::default(),
            }
            .with_source(true)
            .with_website(true)
            .with_wiki(true)
            .with_issues(true)
        }

        pub fn with_options(links: &'a ModLinks, options: DisplayBuilderOptions) -> Self {
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

    impl Display for DisplayBuilder<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let the = self.the_links;
            let no_source = || "No source!";

            writeln!(f, "\nLinks:")?;
            writeln!(f, "Website: {}", the.website().as_str().italic())?;

            writeln!(
                f,
                "Wiki: {}",
                the.wiki_url()
                    .map(Url::as_str)
                    .unwrap_or_else(no_source)
                    .italic()
            )?;

            writeln!(
                f,
                "Issues: {}",
                the.issues_url()
                    .map(Url::as_str)
                    .unwrap_or_else(no_source)
                    .italic()
            )?;

            writeln!(
                f,
                "Source: {}",
                the.source_url()
                    .map(Url::as_str)
                    .unwrap_or_else(no_source)
                    .italic()
            )?;

            Ok(())
        }
    }
}
