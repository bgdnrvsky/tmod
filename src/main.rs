use anyhow::{self, Context};
use reqwest as rq;
use serde::Deserialize;

const TOKEN: &str = "$2a$10$bL4bIL5pUWqfcO7KQtnMReakwtfHbNKh6v1uTpKlzhwoueEJQnPnm";
const GAMES_LIST_URL: &str = "https://api.curseforge.com/v1/games"; // https://github.com/fn2006/PollyMC/wiki/CurseForge-Workaround

#[derive(Debug, Deserialize)]
struct GamesList {
    data: Vec<GameEntry>,
}

impl GamesList {
    fn find_game(&self, game_name: impl AsRef<str>) -> Option<&GameEntry> {
        self.data.iter().find(|entry| {
            entry.get_name() == game_name.as_ref() || entry.get_slug() == game_name.as_ref()
        })
    }
}

#[derive(Debug, Deserialize)]
struct GameEntry {
    id: usize,
    name: String,
    slug: String,
}

impl GameEntry {
    fn get_slug(&self) -> &str {
        &self.name
    }

    fn get_name(&self) -> &str {
        &self.slug
    }

    fn get_id(&self) -> usize {
        self.id
    }
}

fn main() -> anyhow::Result<()> {
    let mut req = rq::blocking::Request::new(
        rq::Method::GET,
        rq::Url::parse(GAMES_LIST_URL)
            .with_context(|| format!("While parsing games list url: {}", GAMES_LIST_URL))?,
    );

    let header_map = req.headers_mut();
    header_map.insert("x-api-key", rq::header::HeaderValue::from_static(TOKEN));

    let client = rq::blocking::Client::new();
    let response = client.execute(req).with_context(|| {
        format!(
            "While retrieving games list from CurseForge via url: {}",
            GAMES_LIST_URL
        )
    })?;

    let games: GamesList = response.json().with_context(|| {
        format!(
            "While decoding games list from CurseForge via url {}",
            GAMES_LIST_URL
        )
    })?;

    let minecraft_id: Option<usize> = games.find_game("minecraft").map(GameEntry::get_id);

    println!("{:?}", minecraft_id);

    Ok(())
}
