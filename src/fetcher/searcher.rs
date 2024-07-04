use std::cell::OnceCell;

use anyhow::Context;

use super::items::*;
use super::AdditionalFetchParameters;
use super::Fetchable;

use super::mod_search::{search_list::ModSearchList, search_mod::SearchedMod};

#[derive(Debug, Default)]
pub struct Searcher {
    minecraft_id: FetchCell<MinecraftId>,
    minecraft_versions: FetchCell<MinecraftVersions>,
    forge_versions: FetchCell<ForgeVersions>,
    curseforge_categories: FetchCell<CurseForgeCategories>,
}

impl Searcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn minecraft_id(&self) -> anyhow::Result<&MinecraftId> {
        self.minecraft_id
            .get_or_fetch(self, |_| Ok(AdditionalFetchParameters::default()))
    }

    pub fn minecraft_versions(&self) -> anyhow::Result<&MinecraftVersions> {
        self.minecraft_versions
            .get_or_fetch(self, |_| Ok(AdditionalFetchParameters::default()))
    }

    pub fn forge_versions(&self) -> anyhow::Result<&ForgeVersions> {
        self.forge_versions
            .get_or_fetch(self, |_| Ok(AdditionalFetchParameters::default()))
    }

    pub fn curseforge_categories(&self) -> anyhow::Result<&CurseForgeCategories> {
        self.curseforge_categories.get_or_fetch(self, |fetcher| {
            let mut params = AdditionalFetchParameters::default();

            params.add_query(("gameId", fetcher.minecraft_id()?.to_string()));
            params.add_query(("classesOnly", "true"));

            Ok(params)
        })
    }

    pub fn search_mod_by_id(&self, id: usize) -> anyhow::Result<SearchedMod> {
        let mut params = AdditionalFetchParameters::default();
        params.add_segment(format!("{id}"));
        SearchedMod::fetch(params)
    }

    pub fn search_mod_by_name(&self, slug: impl AsRef<str>) -> anyhow::Result<ModSearchList> {
        let mut params = AdditionalFetchParameters::default();
        let mods_class = self
            .curseforge_categories()?
            .get("Mods")
            .context("No category `Mods` found")?;

        params.add_query(("gameId", self.minecraft_id()?.to_string()));
        params.add_query(("classId", mods_class.to_string()));
        params.add_query(("slug", slug));

        ModSearchList::fetch(params)
    }
}

#[derive(Debug, Clone, Default)]
struct FetchCell<T> {
    cell: OnceCell<T>,
}

impl<T> FetchCell<T>
where
    T: Fetchable,
{
    fn get_or_fetch<F>(&self, fetcher: &Searcher, f: F) -> anyhow::Result<&T>
    where
        F: FnOnce(&Searcher) -> anyhow::Result<AdditionalFetchParameters>,
    {
        if let Some(item) = self.cell.get() {
            Ok(item)
        } else {
            let item = T::fetch(f(fetcher)?)?;
            let prev = self.cell.set(item);
            debug_assert!(prev.is_ok());
            Ok(self.cell.get().unwrap())
        }
    }
}
