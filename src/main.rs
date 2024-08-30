use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};
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
        display_options: tmod::fetcher::mod_search::search_mod::display::Options,
        #[command(subcommand)]
        subadd: AddTargets,
    },
    /// Remove a mod from the `pool`
    Remove {
        name: String,
        #[arg(value_enum)]
        from: Option<Locations>,
    },
    /// Search a remote mod and print its info
    Info {
        #[clap(flatten)]
        display_options: tmod::fetcher::mod_search::search_mod::display::Options,
        /// And also add the mod to the `pool`
        #[arg(long, default_value_t = false)]
        add_as_well: bool,
        #[command(subcommand)]
        target: SearchTargets,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Locations {
    Remotes,
    Locals,
}

#[derive(Debug, Subcommand)]
enum SearchTargets {
    /// Using CurseForge mod id
    Id { mod_id: usize },
    /// Using mod's 'slug' (slug is not always the same as the mod name)
    Slug { mod_slug: String },
}

#[derive(Debug, Subcommand)]
enum AddTargets {
    #[clap(flatten)]
    Remote(SearchTargets),
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

    match cli.command {
        Commands::Init => {
            Pool::init(&cli.pool_dir)?;
        }
        Commands::Add {
            subadd,
            no_print,
            display_options,
        } => {
            let searcher = Searcher::new();
            let mut pool = Pool::new(&cli.pool_dir)
                .context("Error initializing the pool (maybe you should init it?)")?;

            match subadd {
                AddTargets::Remote(SearchTargets::Id { mod_id }) => {
                    let the_mod = searcher.search_mod_by_id(mod_id)?;

                    if !no_print {
                        print!("{}", the_mod.display_with_options(display_options));
                    }

                    pool.add_to_remotes(&the_mod)?;
                }
                AddTargets::Remote(SearchTargets::Slug { mod_slug }) => {
                    if let Some(the_mod) = searcher.search_mod_by_slug(&mod_slug)? {
                        if !no_print {
                            print!("{}", the_mod.display_with_options(display_options));
                        }

                        pool.add_to_remotes(&the_mod)?;
                    } else {
                        anyhow::bail!("No mod `{mod_slug}` was found");
                    }
                }
                AddTargets::Jar { r#move, path } => {
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
                        println!("Moving {}", path.display());
                        std::fs::remove_file(path).context("Removing jar")?;
                    } else {
                        println!("Copying {}", path.display());
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
        Commands::Remove { from, name } => {
            let mut pool = Pool::new(&cli.pool_dir)
                .context("Error initializing the pool (maybe you should init it?)")?;

            let present = match from {
                Some(loc) => match loc {
                    Locations::Remotes => pool.remove_from_remotes(&name),
                    Locations::Locals => pool.remove_from_locals(&name),
                },
                None => pool.remove_mod(&name),
            }?;

            if !present {
                println!("No mod {} was removed", name.italic().blue());
            }
        }
        Commands::Info {
            display_options,
            target,
            add_as_well,
        } => {
            let searcher = Searcher::new();

            let the_mod = match target {
                SearchTargets::Id { mod_id } => searcher.search_mod_by_id(mod_id)?,
                SearchTargets::Slug { mod_slug } => {
                    if let Some(the_mod) = searcher.search_mod_by_slug(&mod_slug)? {
                        the_mod
                    } else {
                        anyhow::bail!("No mod `{mod_slug}` was found");
                    }
                }
            };

            print!("{}", the_mod.display_with_options(display_options));

            if add_as_well {
                let mut pool = Pool::new(&cli.pool_dir)
                    .context("Error initializing the pool (maybe you should init it?)")?;

                pool.add_to_remotes(&the_mod)?;
            }
        }
    }

    Ok(())
}
