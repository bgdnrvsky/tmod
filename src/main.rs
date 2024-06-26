use clap::{Parser, Subcommand};
use tmod::fetcher::searcher::Searcher;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Add {
        #[command(subcommand)]
        subadd: AddCommandTypes,
    },
}

#[derive(Debug, Subcommand)]
enum AddCommandTypes {
    Url {
        #[arg(value_parser = valid_curse_forge_url)]
        curse_forge_url: url::Url,
    },
}

fn valid_curse_forge_url(s: &str) -> Result<url::Url, String> {
    let url = url::Url::parse(s).map_err(|e| e.to_string())?;

    if !url
        .host_str()
        .is_some_and(|host| host == "www.curseforge.com")
    {
        return Err(String::from(
            "The url's host is expected to be `www.curseforge.com`",
        ));
    }

    if let Some(mut segments) = url.path_segments() {
        if !segments.next().is_some_and(|seg| seg == "minecraft") {
            return Err(String::from(
                "First in curseforge url should be `minecraft`",
            ));
        }

        if !segments.next().is_some_and(|seg| seg == "mc-mods") {
            return Err(String::from(
                "Second segment in curseforge url should be `mc-mods`",
            ));
        }

        if segments.next().is_none() {
            return Err(String::from("Missing mod name in segments"));
        }
    } else {
        return Err(String::from(
            "The url's path segments didn't match the expected `/minecraft/mc-mods/MOD_NAME`",
        ));
    }

    Ok(url)
}

fn extract_mod_name_from_url(url: &url::Url) -> &str {
    // Given that `valid_curse_forge_url` didn't fail, no need for checking anymore
    url.path_segments().unwrap().nth(2).unwrap()
}

fn main() -> anyhow::Result<()> {
    let cli = Args::parse();
    let searcher = Searcher::new();

    match cli.command {
        Commands::Add { subadd } => match subadd {
            AddCommandTypes::Url { curse_forge_url } => {
                let mod_name = extract_mod_name_from_url(&curse_forge_url);
                let the_mod = searcher.search_mod_by_name(mod_name)?;

                println!("{the_mod}");
            }
        },
    }

    Ok(())
}
