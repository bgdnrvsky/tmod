use std::{collections::HashMap, fmt::Display};

use anyhow::Context;
use serde::Deserialize;

use crate::version::SingleVersion;

use super::{rq::*, Fetchable, Url};

/// Example JSON:
/// ```json
/// {
///   "data": [
///     {
///       "id": 0,
///       "name": "string",
///       "slug": "string",
///       "dateModified": "2019-08-24T14:15:22Z",
///       "assets": {
///         "iconUrl": "string",
///         "tileUrl": "string",
///         "coverUrl": "string"
///       },
///       "status": 1,
///       "apiStatus": 1
///     }
///   ],
///   "pagination": {
///     "index": 0,
///     "pageSize": 0,
///     "resultCount": 0,
///     "totalCount": 0
///   }
/// }
/// ```
pub type MinecraftId = usize;

impl Fetchable for MinecraftId {
    fn link() -> Url {
        Url::parse("https://api.curseforge.com/v1/games").unwrap()
    }

    fn parse(response: Response) -> anyhow::Result<Self> {
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
            .find(|entry| entry.name == "minecraft" || entry.slug == "minecraft")
            .map(|entry| entry.id)
            .context("Minecraft was not found in the list of games")
    }

    fn info() -> impl Display {
        "Getting Minecraft id from CurseForge"
    }
}

/// Example JSON:
/// ```json
/// {
///     "result":
///     [
///         "1.20.1",
///         "1.20",
///         "1.19.4",
///         "1.19.3",
///         "1.19.2",
///         "1.19.1",
///         "1.19",
///         "..."
///     ]
/// }
/// ```
pub type MinecraftVersions = Vec<SingleVersion>;

impl Fetchable for MinecraftVersions {
    fn link() -> Url {
        Url::parse("https://mc-versions-api.net/api/java").unwrap()
    }

    fn parse(response: Response) -> anyhow::Result<Self> {
        #[derive(Debug, Clone, Deserialize)]
        struct Data {
            result: Vec<SingleVersion>,
        }

        response
            .into_json::<Data>()
            .context("Parsing Minecraft versions")
            .map(|v| v.result)
    }

    fn info() -> impl Display {
        "Getting Minecraft versions"
    }
}

/// Mapping from a Minecraft version to available Forge versions
///
/// Example JSON:
/// ```json
///{
///  "result": [
///    {
///     "1.21": [
///       "51.0.21",
///       "51.0.18",
///       "51.0.17",
///       "51.0.16",
///       "51.0.15",
///       "51.0.13",
///       "51.0.8",
///       "51.0.7",
///       "51.0.6",
///       "51.0.5",
///       "51.0.4",
///       "51.0.3",
///       "51.0.1",
///       "51.0.0"
///      ]
///    }
///  ]
///}
/// ```
pub type ForgeVersions = HashMap<SingleVersion, Vec<SingleVersion>>;

impl Fetchable for ForgeVersions {
    fn link() -> Url {
        Url::parse("https://mc-versions-api.net/api/forge").unwrap()
    }

    fn parse(response: Response) -> anyhow::Result<Self> {
        #[derive(Debug, Clone, Deserialize)]
        struct Data {
            result: [HashMap<SingleVersion, Vec<SingleVersion>>; 1],
        }

        response
            .into_json::<Data>()
            .context("Deserializing Forge versions")
            .map(|Data { result: [version] }| version)
    }

    fn info() -> impl Display {
        "Getting Forge versions from CurseForge"
    }
}

#[derive(Debug, Default)]
pub struct FabricVersions(pub Vec<SingleVersion>);

impl Fetchable for FabricVersions {
    fn link() -> Url {
        Url::parse("https://meta.fabricmc.net/v2/versions/loader").unwrap()
    }

    fn parse(response: Response) -> anyhow::Result<Self> {
        #[derive(Debug, Clone, Deserialize)]
        struct Item {
            version: SingleVersion,
        }

        response
            .into_json::<Vec<Item>>()
            .context("Deserializing Fabric versions")
            .map(|items| items.into_iter().map(|item| item.version).collect())
            .map(Self)
    }

    fn info() -> impl Display {
        "Getting Fabric versions"
    }
}

/// Example JSON:
/// ```json
/// {
///   "data": [
///     {
///       "id": 0,
///       "gameId": 0,
///       "name": "string",
///       "slug": "string",
///       "url": "string",
///       "iconUrl": "string",
///       "dateModified": "2019-08-24T14:15:22Z",
///       "isClass": true,
///       "classId": 0,
///       "parentCategoryId": 0,
///       "displayIndex": 0
///     }
///   ]
/// }
/// ```
pub type CurseForgeCategories = HashMap<String, usize>;

impl Fetchable for CurseForgeCategories {
    fn link() -> Url {
        Url::parse("https://api.curseforge.com/v1/categories").unwrap()
    }

    fn parse(response: Response) -> anyhow::Result<Self> {
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

        Ok(data
            .into_iter()
            .map(|entry| (entry.name, entry.id))
            .collect())
    }

    fn info() -> impl Display {
        "Getting game categories from CurseForge"
    }
}
