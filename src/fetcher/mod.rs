#![allow(clippy::borrow_interior_mutable_const)]
pub mod mod_search;

use std::cell::LazyCell;
use std::collections::HashMap;
use std::sync::OnceLock;

use anyhow::Context;
use chrono::{DateTime, Utc};
use loading::{Loading, Spinner};
use rq::Response;
use serde::Deserialize;
use ureq as rq;
use url::Url;

use crate::pool::config::Config;
use mod_search::{
    search_list::ModSearchList,
    search_mod::{ModFile, SearchedMod},
};

pub const TOKEN: &str = "$2a$10$bL4bIL5pUWqfcO7KQtnMReakwtfHbNKh6v1uTpKlzhwoueEJQnPnm"; // https://github.com/fn2006/PollyMC/wiki/CurseForge-Workaround
#[allow(clippy::declare_interior_mutable_const)]
const API_URL: LazyCell<Url> =
    LazyCell::new(|| Url::parse("https://api.curseforge.com/v1").unwrap());
pub static SEARCHER: Searcher = Searcher::new();

#[derive(Debug, Default)]
pub struct Searcher {
    silent: OnceLock<bool>,
    minecraft_id: OnceLock<usize>,
    minecraft_versions: OnceLock<Vec<String>>,
    fabric_versions: OnceLock<Vec<String>>,
    curseforge_categories: OnceLock<HashMap<String, usize>>,
}

impl Searcher {
    pub const fn new() -> Self {
        Self {
            silent: OnceLock::new(),
            minecraft_id: OnceLock::new(),
            minecraft_versions: OnceLock::new(),
            fabric_versions: OnceLock::new(),
            curseforge_categories: OnceLock::new(),
        }
    }

    pub fn is_silent(&self) -> bool {
        *self.silent.get_or_init(Default::default)
    }

    pub fn set_silent(&self, silent: bool) {
        let _ = self.silent.set(silent);
    }

    pub fn minecraft_id(&self) -> anyhow::Result<usize> {
        if self.minecraft_id.get().is_none() {
            let mut url = API_URL.clone();
            url.path_segments_mut().unwrap().push("games");

            let response = FetchParameters::new(url, self.is_silent())
                .with_info("Getting Minecraft id")
                .fetch()
                .context("Fetching Minecraft id")?;

            let id = {
                #[derive(Debug, Deserialize)]
                struct GamesList {
                    data: Vec<GameEntry>,
                }

                #[derive(Debug, Deserialize)]
                struct GameEntry {
                    id: usize,
                    name: String,
                    slug: String,
                }

                let games: GamesList = response.into_json()?;

                games
                    .data
                    .into_iter()
                    .find(|entry| entry.slug == "minecraft" || entry.name == "minecraft")
                    .map(|entry| entry.id)
                    .context("Minecraft was not found in the list of games")
            }?;

            self.minecraft_id.set(id).unwrap();
        }

        Ok(self.minecraft_id.get().copied().unwrap())
    }

    pub fn minecraft_versions(&self) -> anyhow::Result<&[String]> {
        if self.minecraft_versions.get().is_none() {
            let url = Url::parse("https://mc-versions-api.net/api/java").unwrap();
            let response = FetchParameters::new(url, self.is_silent())
                .with_info("Getting Minecraft versions")
                .fetch()
                .context("Fetching Minecraft versions")?;

            let versions = {
                #[derive(Debug, Clone, Deserialize)]
                struct Data {
                    result: Vec<String>,
                }

                response
                    .into_json::<Data>()
                    .context("Parsing Minecraft versions")
                    .map(|v| v.result)
            }?;

            self.minecraft_versions.set(versions).unwrap();
        }

        Ok(self.minecraft_versions.get().unwrap())
    }

    pub fn fabric_versions(&self) -> anyhow::Result<&[String]> {
        if self.fabric_versions.get().is_none() {
            let url = Url::parse("https://meta.fabricmc.net/v2/versions/loader").unwrap();
            let response = FetchParameters::new(url, self.is_silent())
                .with_info("Getting Fabric versions")
                .fetch()
                .context("Fetching Fabric versions")?;

            let versions = {
                #[derive(Debug, Clone, Deserialize)]
                struct Item {
                    version: String,
                }

                response
                    .into_json::<Vec<Item>>()
                    .context("Deserializing Fabric versions")
                    .map(|items| items.into_iter().map(|item| item.version).collect())
            }?;

            self.fabric_versions.set(versions).unwrap();
        }

        Ok(self.fabric_versions.get().unwrap())
    }

    pub fn curseforge_categories(&self) -> anyhow::Result<&HashMap<String, usize>> {
        if self.curseforge_categories.get().is_none() {
            let mut url = API_URL.clone();
            url.path_segments_mut().unwrap().push("categories");
            url.query_pairs_mut()
                .append_pair("gameId", &self.minecraft_id()?.to_string())
                .append_pair("classesOnly", "true");

            let response = FetchParameters::new(url, self.is_silent())
                .with_info("Getting game categories")
                .fetch()
                .context("Fetching game categories")?;

            let categories = {
                #[derive(Debug, Clone, Deserialize)]
                struct Data {
                    data: Vec<CategoryEntry>,
                }

                #[derive(Debug, Clone, Deserialize)]
                struct CategoryEntry {
                    name: String,
                    id: usize,
                }

                let data = response.into_json::<Data>()?.data;

                data.into_iter()
                    .map(|entry| (entry.name, entry.id))
                    .collect()
            };

            self.curseforge_categories.set(categories).unwrap();
        }

        Ok(self.curseforge_categories.get().unwrap())
    }

    pub fn search_mod_by_id(&self, id: usize) -> anyhow::Result<SearchedMod> {
        let mut url = API_URL.clone();
        url.path_segments_mut()
            .unwrap()
            .push("mods")
            .push(id.to_string().as_str());

        let response = FetchParameters::new(url, self.is_silent())
            .with_info(format!("Getting Minecraft mod by id ({id})"))
            .fetch()
            .with_context(|| format!("Fetching mod {id}"))?;

        #[derive(Debug, Clone, Deserialize)]
        struct Data {
            data: SearchedMod,
        }

        response
            .into_json::<Data>()
            .context("Deserializing response")
            .map(|data| data.data)
    }

    pub fn search_mod_by_slug(&self, slug: impl AsRef<str>) -> anyhow::Result<SearchedMod> {
        let mods_category = self
            .curseforge_categories()?
            .get("Mods")
            .context("No category 'Mods' found")?;

        let mut url = API_URL.clone();
        url.path_segments_mut().unwrap().push("mods").push("search");
        url.query_pairs_mut()
            .append_pair("gameId", self.minecraft_id()?.to_string().as_str())
            .append_pair("classId", mods_category.to_string().as_str())
            .append_pair("slug", slug.as_ref());

        let response = FetchParameters::new(url, self.is_silent())
            .with_info(format!("Searching for the mod '{}'", slug.as_ref()))
            .fetch()
            .with_context(|| format!("Fetching the mod '{}'", slug.as_ref()))?;

        let list: ModSearchList = response
            .into_json()
            .context("Deserializing searched mods")?;

        match list.to_single_mod() {
            Ok(r#mod) => Ok(r#mod),
            Err(0) => anyhow::bail!("The mod '{}' is not found", slug.as_ref()),
            Err(n) => anyhow::bail!("The list should have contained 1 mod, but found {n}"),
        }
    }

    /// If the timestamp is Some, searches for the file with the same timestamp,
    /// if None, gets the latest file published
    pub fn get_specific_mod_file(
        &self,
        the_mod: &SearchedMod,
        config: &Config,
        timestamp: Option<DateTime<Utc>>,
    ) -> anyhow::Result<ModFile> {
        let files = Self::get_mod_files(self, the_mod, config)?;

        if let Some(time) = timestamp {
            files
                .into_iter()
                .find(|file| file.date == time)
                .with_context(|| {
                    format!(
                        "The file with specified timestamp ({}) wasn't found in files",
                        time
                    )
                })
        } else {
            files
                .into_iter()
                .max_by_key(|file| file.date)
                .context("Fetched files contains 0 files")
        }
    }

    pub fn get_mod_files(
        &self,
        the_mod: &SearchedMod,
        config: &Config,
    ) -> anyhow::Result<Vec<ModFile>> {
        let mut url = API_URL.clone();
        url.path_segments_mut()
            .unwrap()
            .push("mods")
            .push(the_mod.id.to_string().as_str())
            .push("files");
        url.query_pairs_mut()
            .append_pair("gameVersion", config.game_version.to_string().as_str())
            .append_pair("modLoaderType", (config.loader as u8).to_string().as_str());

        let mut files = FetchParameters::new(url, self.is_silent())
            .with_info(format!("Getting mod files for '{}'", the_mod.slug).as_str())
            .fetch()
            .with_context(|| format!("Fetching mod files for '{}'", the_mod.slug))?
            .into_json::<Data>()
            .context("Deserializing mod files")
            .map(|result| result.data)?;

        #[derive(Deserialize)]
        struct Data {
            data: Vec<ModFile>,
        }

        // Filter relations that are useless
        // Only keep relations that are either required dependencies or incompatible
        for file in files.iter_mut() {
            file.relations.retain(|dep| dep.relation.is_needed());
        }

        Ok(files)
    }

    pub fn download_file(&self, mod_file: &ModFile) -> anyhow::Result<Response> {
        let info = format!("Downloading the mod from {}", mod_file.url);
        FetchParameters::new(mod_file.url.clone(), self.is_silent())
            .with_info(info.clone())
            .fetch()
            .with_context(|| info)
    }
}

#[derive(Debug, Clone)]
struct FetchParameters {
    url: Url,
    info: Option<String>,
    silent: bool,
}

impl FetchParameters {
    fn new(url: Url, silent: bool) -> Self {
        Self {
            url,
            info: None,
            silent,
        }
    }

    fn with_info(mut self, info: impl std::fmt::Display) -> Self {
        self.info = Some(info.to_string());
        self
    }

    fn fetch(self) -> anyhow::Result<Response> {
        let loading_info = self.info.unwrap_or_else(|| String::from("Fetching data"));
        let loading = if self.silent {
            None
        } else {
            let loading = Loading::new(Spinner::default());
            loading.info(&loading_info);
            loading.text("Fetching");

            Some(loading)
        };

        let response = rq::get(self.url.as_str())
            .set("x-api-key", TOKEN)
            .call()
            .context("Getting a response from CurseForge")?;

        if let Some(loading) = loading {
            loading.end()
        }

        if response.status() != 200 {
            anyhow::bail!(
                "{info} from {url} failed with status {status} ({status_text})",
                status = response.status(),
                status_text = response.status_text(),
                info = loading_info,
                url = response.get_url() // The actual URL can be different from the one we requested
            );
        }

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use crate::pool::loader::Loaders;

    use super::*;

    #[test]
    fn mod_by_id() -> anyhow::Result<()> {
        let searcher = Searcher::new(true);

        let alexs_mobs = searcher.search_mod_by_id(426558)?;
        assert_eq!(alexs_mobs.slug, "alexs-mobs");

        let jei = searcher.search_mod_by_id(238222)?;
        assert_eq!(jei.slug, "jei");

        Ok(())
    }

    #[test]
    fn minecraft_id() -> anyhow::Result<()> {
        Searcher::new(true).minecraft_id()?;

        Ok(())
    }

    #[test]
    #[ignore = "might be very long (~15 secs)"]
    fn minecraft_versions() -> anyhow::Result<()> {
        let searcher = Searcher::new(true);
        let versions = searcher.minecraft_versions()?;
        assert!(!versions.is_empty());

        Ok(())
    }

    #[test]
    fn fabric_versions() -> anyhow::Result<()> {
        let searcher = Searcher::new(true);
        let versions = searcher.fabric_versions()?;
        assert!(!versions.is_empty());

        Ok(())
    }

    #[test]
    fn curseforge_categories() -> anyhow::Result<()> {
        let searcher = Searcher::new(true);
        let categories = searcher.curseforge_categories()?;

        assert!(!categories.is_empty());
        assert!(categories.contains_key("Mods"));

        Ok(())
    }

    #[test]
    fn mod_by_slug() -> anyhow::Result<()> {
        let searcher = Searcher::new(true);

        let alexs_mobs = searcher.search_mod_by_slug("alexs-mobs")?;
        assert_eq!(alexs_mobs.id, 426558);

        let jei = searcher.search_mod_by_slug("jei")?;
        assert_eq!(jei.id, 238222);

        assert!(searcher.search_mod_by_slug("alexs_mobs").is_err());

        Ok(())
    }

    #[test]
    fn mod_files() -> anyhow::Result<()> {
        let searcher = Searcher::new(true);
        let config = Config::new(Loaders::Forge, String::from("1.20.1"));
        let alexs_mobs = searcher.search_mod_by_id(426558)?;

        let files = searcher.get_mod_files(&alexs_mobs, &config)?;
        assert!(
            !files.is_empty(),
            "Since the mod 'alexs-mobs' is compatible, we should get some files"
        );

        Ok(())
    }
}
