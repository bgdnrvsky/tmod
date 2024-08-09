use clap::{Parser, Subcommand};
use colored::Colorize;
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
    /// By CurseForge mod id
    Id { mod_id: usize },
    /// Using mod's 'slug'
    Slug { mod_slug: String },
}

fn main() -> anyhow::Result<()> {
    let cli = Args::parse();
    let searcher = Searcher::new();

    match cli.command {
        Commands::Add { subadd } => match subadd {
            AddCommandTypes::Id { mod_id } => {
                let the_mod = searcher.search_mod_by_id(mod_id)?;

                print!("{}", the_mod.display());
            }
            AddCommandTypes::Slug { mod_slug } => {
                if let Some(the_mod) = searcher.search_mod_by_slug(&mod_slug)? {
                    print!("{}", the_mod.display());
                } else {
                    println!("No mod {slug} found", slug = mod_slug.italic().blue());
                }
            }
        },
    }

    Ok(())
}
