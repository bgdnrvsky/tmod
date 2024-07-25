use anyhow::{bail, ensure};
use clap::{Parser, Subcommand};
use tmod::fetcher::searcher::Searcher;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Add minecraft mod to the `pool`
    Add {
        #[command(subcommand)]
        subadd: AddCommandTypes,
    },
}

#[derive(Debug, Subcommand)]
enum AddCommandTypes {
    /// By CurseForge link
    Url {
        #[arg(value_parser = valid_curse_forge_url)]
        curse_forge_url: url::Url,
    },
    /// By CurseForge mod id
    Id { mod_id: usize },
    /// Using mod's 'slug'
    Slug { mod_slug: String },
}

fn valid_curse_forge_url(s: &str) -> anyhow::Result<url::Url> {
    let url = url::Url::parse(s)?;

    ensure!(
        url.host_str()
            .is_some_and(|host| host == "www.curseforge.com"),
        "The url's host is expected to be `www.curseforge.com`"
    );

    if let Some(mut segments) = url.path_segments() {
        ensure!(
            segments.next().is_some_and(|seg| seg == "minecraft"),
            "First in curseforge url should be `minecraft`"
        );
        ensure!(
            segments.next().is_some_and(|seg| seg == "mc-mods"),
            "Second segment in curseforge url should be `mc-mods`"
        );
        ensure!(segments.next().is_some(), "Missing mod name in segments");
    } else {
        bail!("The url's path segments didn't match the expected `/minecraft/mc-mods/MOD_NAME`")
    }

    Ok(url)
}

fn main() -> anyhow::Result<()> {
    let cli = Args::parse();
    let searcher = Searcher::new();

    match cli.command {
        Commands::Add { subadd } => match subadd {
            AddCommandTypes::Url { curse_forge_url } => {
                let mod_name = curse_forge_url
                    .path_segments()
                    .and_then(|mut segs| segs.nth(2))
                    .expect("Given that `valid_curse_forge_url` didn't fail, no need for checking anymore");

                let mod_list = searcher.search_mod_by_name(mod_name)?;

                print!("{}", mod_list.display());
            }
            AddCommandTypes::Id { mod_id } => {
                let the_mod = searcher.search_mod_by_id(mod_id)?;

                print!("{}", the_mod.display());
            }
            AddCommandTypes::Slug { mod_slug } => {
                let mod_list = searcher.search_mod_by_name(mod_slug)?;

                print!("{}", mod_list.display());
            }
        },
    }

    Ok(())
}
