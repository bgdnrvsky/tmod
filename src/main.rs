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
        /// Do not print the mod to stdout
        #[arg(long, default_value_t = false)]
        no_print: bool,
        #[clap(flatten)]
        display_options:
            tmod::fetcher::mod_search::search_mod::display_builder::DisplayBuilderOptions,
        #[command(subcommand)]
        subadd: AddCommandTypes,
    },
}

#[derive(Debug, Subcommand)]
enum AddCommandTypes {
    /// By CurseForge mod id
    Id { mod_id: usize },
    /// Using mod's 'slug' (slug is not always the same as the mod name)
    Slug { mod_slug: String },
}

fn main() -> anyhow::Result<()> {
    let cli = Args::parse();
    let searcher = Searcher::new();

    match cli.command {
        Commands::Add {
            subadd,
            no_print,
            display_options,
        } => {
            let the_mod = match subadd {
                AddCommandTypes::Id { mod_id } => searcher.search_mod_by_id(mod_id)?,
                AddCommandTypes::Slug { mod_slug } => {
                    if let Some(the_mod) = searcher.search_mod_by_slug(&mod_slug)? {
                        the_mod
                    } else {
                        anyhow::bail!("No mod `{mod_slug}` was found");
                    }
                }
            };

            if !no_print {
                print!("{}", the_mod.display_with_options(display_options));
            }
        }
    }

    Ok(())
}
