pub mod mod_search;

use std::cell::OnceCell;
use std::collections::HashMap;

use anyhow::Context;
use loading::{Loading, Spinner};
use rq::Response;
use serde::Deserialize;
use ureq as rq;
use url::Url;

use crate::{pool::config::Config, version::SingleVersion};
use mod_search::{
    search_list::ModSearchList,
    search_mod::{ModFile, SearchedMod},
};

pub const TOKEN: &str = "$2a$10$bL4bIL5pUWqfcO7KQtnMReakwtfHbNKh6v1uTpKlzhwoueEJQnPnm"; // https://github.com/fn2006/PollyMC/wiki/CurseForge-Workaround

#[derive(Debug, Default)]
pub struct Searcher {
    silent: bool,
    minecraft_id: OnceCell<usize>,
    minecraft_versions: OnceCell<Vec<SingleVersion>>,
    forge_versions: OnceCell<HashMap<SingleVersion, Vec<SingleVersion>>>,
    fabric_versions: OnceCell<Vec<SingleVersion>>,
    curseforge_categories: OnceCell<HashMap<String, usize>>,
}

impl Searcher {
    pub fn new(silent: bool) -> Self {
        Self {
            silent,
            ..Default::default()
        }
    }

    pub fn minecraft_id(&self) -> anyhow::Result<usize> {
        if self.minecraft_id.get().is_none() {
            let url = Url::parse("https://api.curseforge.com/v1/games").unwrap();
            let response = FetchParameters::new(url)
                .silent(self.silent)
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

    pub fn minecraft_versions(&self) -> anyhow::Result<&[SingleVersion]> {
        if self.minecraft_versions.get().is_none() {
            let url = Url::parse("https://mc-versions-api.net/api/java").unwrap();
            let response = FetchParameters::new(url)
                .silent(self.silent)
                .with_info("Getting Minecraft versions")
                .fetch()
                .context("Fetching Minecraft versions")?;

            let versions = {
                #[derive(Debug, Clone, Deserialize)]
                struct Data {
                    result: Vec<SingleVersion>,
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

    pub fn forge_versions(&self) -> anyhow::Result<&HashMap<SingleVersion, Vec<SingleVersion>>> {
        if self.forge_versions.get().is_none() {
            let url = Url::parse("https://mc-versions-api.net/api/forge").unwrap();
            let response = FetchParameters::new(url)
                .silent(self.silent)
                .with_info("Getting Forge versions")
                .fetch()
                .context("Fetching Forge versions")?;

            let versions = {
                #[derive(Debug, Clone, Deserialize)]
                struct Data {
                    result: [HashMap<SingleVersion, Vec<SingleVersion>>; 1],
                }

                response
                    .into_json::<Data>()
                    .context("Deserializing Forge versions")
                    .map(|Data { result: [version] }| version)
            }?;

            self.forge_versions.set(versions).unwrap();
        }

        Ok(self.forge_versions.get().unwrap())
    }

    pub fn fabric_versions(&self) -> anyhow::Result<&[SingleVersion]> {
        if self.fabric_versions.get().is_none() {
            let url = Url::parse("https://meta.fabricmc.net/v2/versions/loader").unwrap();
            let response = FetchParameters::new(url)
                .silent(self.silent)
                .with_info("Getting Fabric versions")
                .fetch()
                .context("Fetching Fabric versions")?;

            let versions = {
                #[derive(Debug, Clone, Deserialize)]
                struct Item {
                    version: SingleVersion,
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
            let url = Url::parse("https://api.curseforge.com/v1/categories").unwrap();
            let response = FetchParameters::new(url)
                .silent(self.silent)
                .add_query("gameId", &self.minecraft_id()?.to_string())
                .add_query("classesOnly", "true")
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
        let url = Url::parse("https://api.curseforge.com/v1/mods").unwrap();
        let response = FetchParameters::new(url)
            .silent(self.silent)
            .with_info(format!("Getting Minecraft mod by id ({id})"))
            .with_segment(id)
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

    pub fn search_mod_by_slug(&self, slug: impl AsRef<str>) -> anyhow::Result<Option<SearchedMod>> {
        let mods_category = self
            .curseforge_categories()?
            .get("Mods")
            .context("No category 'Mods' found")?;

        let url = Url::parse("https://api.curseforge.com/v1/mods/search").unwrap();
        let response = FetchParameters::new(url)
            .add_query("gameId", self.minecraft_id()?.to_string().as_str())
            .add_query("classId", mods_category.to_string().as_str())
            .add_query("slug", slug.as_ref())
            .with_info(format!("Searching for the mod '{}'", slug.as_ref()))
            .fetch()
            .with_context(|| format!("Fetching the mod '{}'", slug.as_ref()))?;

        let list: ModSearchList = response
            .into_json()
            .context("Deserializing searched mods")?;

        match list.to_single_mod() {
            Ok(r#mod) => Ok(Some(r#mod)),
            Err(0) => Ok(None),
            Err(n) => anyhow::bail!("The list should have contained 1 mod, but found {n}"),
        }
    }

    pub fn get_mod_files(
        &self,
        the_mod: &SearchedMod,
        config: &Config,
    ) -> anyhow::Result<Vec<ModFile>> {
        let url = Url::parse(
            format!(
                "https://api.curseforge.com/v1/mods/{id}/files",
                id = the_mod.id()
            )
            .as_str(),
        )
        .unwrap();
        let response = FetchParameters::new(url)
            .silent(self.silent)
            .add_query("gameVersion", config.game_version().to_string().as_str())
            .add_query(
                "modLoaderType",
                (config.loader().kind() as u8).to_string().as_str(),
            )
            .with_info(format!("Getting mod files for '{}'", the_mod.slug()).as_str())
            .fetch()
            .with_context(|| format!("Fetching mod files for '{}'", the_mod.slug()))?;

        #[derive(Deserialize)]
        struct Data {
            data: Vec<ModFile>,
        }

        response
            .into_json::<Data>()
            .context("Deserializing mod files")
            .map(|result| result.data)
    }
}

#[derive(Debug, Clone)]
struct FetchParameters {
    url: Url,
    info: Option<String>,
    silent: bool,
}

impl FetchParameters {
    fn new(url: Url) -> Self {
        Self {
            url,
            info: None,
            silent: false,
        }
    }

    fn with_info(mut self, info: impl std::fmt::Display) -> Self {
        self.info = Some(info.to_string());
        self
    }

    fn add_query(mut self, name: &str, value: &str) -> Self {
        self.url.query_pairs_mut().append_pair(name, value);
        self
    }

    fn silent(mut self, silent: bool) -> Self {
        self.silent = silent;
        self
    }

    fn fetch(self) -> anyhow::Result<Response> {
        let loading = if self.silent {
            None
        } else {
            let loading = Loading::new(Spinner::default());
            loading.info(self.info.unwrap_or_else(|| String::from("Fetching data")));
            loading.text("Fetching");

            Some(loading)
        };

        let response = rq::get(self.url.as_str())
            .set("x-api-key", TOKEN)
            .call()
            .context("Getting a response from CurseForge");

        if let Some(loading) = loading {
            loading.end()
        }

        response
    }

    fn with_segment(mut self, value: impl std::fmt::Display) -> Self {
        self.url
            .path_segments_mut()
            .expect("The url can be a base")
            .push(value.to_string().as_str());
        self
    }
}

#[cfg(not(feature = "offline"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mod_by_id() -> anyhow::Result<()> {
        let searcher = Searcher::new(true);

        let alexs_mobs = searcher.search_mod_by_id(426558)?;
        assert_eq!(alexs_mobs.slug(), "alexs-mobs");

        let jei = searcher.search_mod_by_id(238222)?;
        assert_eq!(jei.slug(), "jei");

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
    #[ignore = "might be very long (~15 secs)"]
    fn forge_versions() -> anyhow::Result<()> {
        let searcher = Searcher::new(true);
        let versions = searcher.forge_versions()?;
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
}
