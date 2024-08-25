use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
use colored::Colorize;
use jars::{jar, JarOption};
use tmod::{fetcher::searcher::Searcher, jar::JarMod, pool::Pool};

#[derive(Parser)]
struct Cli {
    #[arg(long, default_value = ".tmod", value_name = "PATH")]
    pool_dir: PathBuf,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Initialize a new `pool`
    Init,
    /// List the mods in the `pool`
    List,
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
    /// Add your own jar file
    Jar {
        /// Move the file instead of default copying
        #[arg(short, long, default_value_t = false)]
        r#move: bool,
        path: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let searcher = Searcher::new();

    match cli.command {
        Commands::Init => {
            Pool::init(&cli.pool_dir)?;
        }
        Commands::Add {
            subadd,
            no_print,
            display_options,
        } => {
            let mut pool = Pool::new(&cli.pool_dir)
                .context("Error initializing the pool (maybe you should init it?)")?;

            match subadd {
                AddCommandTypes::Id { mod_id } => {
                    let the_mod = searcher.search_mod_by_id(mod_id)?;

                    if !no_print {
                        print!("{}", the_mod.display_with_options(display_options));
                    }

                    pool.add_to_remotes(&the_mod)?;
                }
                AddCommandTypes::Slug { mod_slug } => {
                    if let Some(the_mod) = searcher.search_mod_by_slug(&mod_slug)? {
                        if !no_print {
                            print!("{}", the_mod.display_with_options(display_options));
                        }

                        pool.add_to_remotes(&the_mod)?;
                    } else {
                        anyhow::bail!("No mod `{mod_slug}` was found");
                    }
                }
                AddCommandTypes::Jar { r#move, path } => {
                    if path.extension().is_none()
                        || path.extension().is_some_and(|ext| ext != "jar")
                    {
                        eprintln!("WARNING: The file you provided doesn't seem like a jar");
                    }

                    let jar = jar(&path, JarOption::default())
                        .context("Opening jar")
                        .and_then(JarMod::try_from)
                        .context("Reading jar")?;

                    println!(
                        "Jar info: name - {}, deps count - {}, incomps count - {}",
                        jar.name().blue().italic(),
                        jar.dependencies().len(),
                        jar.incompatibilities().len()
                    );

                    if r#move {
                        println!("Moving {}", path.to_string_lossy());
                        std::fs::remove_file(path).context("Removing jar")?;
                    } else {
                        println!("Copying {}", path.to_string_lossy());
                    }

                    pool.add_to_locals(jar).context("Adding to locals")?;
                }
            };
        }
        Commands::List => {
            let pool = Pool::new(&cli.pool_dir)
                .context("Error initializing the pool (maybe you should init it?)")?;

            let remotes = pool.remotes();
            let locals = pool.locals();

            println!("Remotes ({} mod(s)):", remotes.len());
            for r in remotes {
                println!("\t- {}", r.italic().blue());
            }

            println!("Locals ({} mod(s)):", locals.len());
            for l in locals {
                println!("\t- {}", l.name().italic().blue());
            }
        }
    }

    Ok(())
}
