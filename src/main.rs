use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
use colored::Colorize;
use ptree::TreeBuilder;
use tmod::{
    fetcher::{
        mod_search::search_mod::{display::ModOptions, SearchedMod},
        Searcher, SEARCHER,
    },
    jar::JarMod,
    pool::{config::Config, Pool},
};

#[derive(Parser)]
struct Cli {
    #[arg(long, default_value = ".tmod", value_name = "PATH")]
    pool_dir: PathBuf,
    #[command(subcommand)]
    command: Commands,
    /// Dot not print Tmod log messages
    #[arg(short, long, default_value_t = false)]
    quiet: bool,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Initialize a new `pool`
    Init,
    /// List the mods in the `pool`
    List,
    /// Add minecraft mod to the `pool`
    Add {
        /// Force add the mod
        #[arg(short, long, default_value_t = false)]
        force: bool,
        #[clap(flatten)]
        display_options: ModOptions,
        #[command(subcommand)]
        subadd: SearchTargets,
        /// When adding a Jar, move the file instead of copying
        #[arg(short, long, default_value_t = false)]
        r#move: bool,
    },
    /// Remove a mod from the `pool`
    Remove {
        #[arg(required = true)]
        names: Vec<String>,
    },
    /// Search a remote mod and print its info
    Info {
        #[clap(flatten)]
        display_options: ModOptions,
        #[command(subcommand)]
        target: SearchTargets,
    },
    Tree,
}

#[derive(Debug, Subcommand)]
enum SearchTargets {
    /// Using CurseForge mod id
    Id {
        #[arg(required = true)]
        mod_ids: Vec<usize>,
    },
    /// Using mod's 'slug' (slug is not always the same as the mod name)
    Slug {
        #[arg(required = true)]
        mod_slugs: Vec<String>,
    },
    /// Using your local Jar file
    Jar {
        #[arg(required = true)]
        paths: Vec<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut searcher = SEARCHER.lock().unwrap();
    searcher.set_silent(cli.quiet);

    match cli.command {
        Commands::Init => {
            Pool::init(&cli.pool_dir)?;
        }
        Commands::Add {
            subadd,
            display_options,
            force,
            r#move,
        } => {
            let mut pool = Pool::new(&cli.pool_dir).context("Error initializing the pool")?;

            let mods: Vec<anyhow::Result<_>> = match subadd {
                SearchTargets::Id { mod_ids } => mod_ids
                    .into_iter()
                    .map(|id| searcher.search_mod_by_id(id))
                    .collect(),
                SearchTargets::Slug { mod_slugs } => mod_slugs
                    .into_iter()
                    .map(|slug| searcher.search_mod_by_slug(&slug))
                    .collect(),
                SearchTargets::Jar { paths } => {
                    for path in paths {
                        match JarMod::open(&path) {
                            Ok(jar) => {
                                if r#move {
                                    std::fs::remove_file(&path).context("Removing jar")?;
                                    if !cli.quiet {
                                        println!("Moving {}", path.display());
                                    }
                                } else if !cli.quiet {
                                    println!("Copying {}", path.display());
                                }

                                pool.add_to_locals(jar);
                            }
                            Err(e) => eprintln!("Error adding local mod: {e}"),
                        }
                    }

                    pool.save().context("Saving the pool")?;

                    return Ok(());
                }
            };

            for the_mod in mods {
                match the_mod {
                    Ok(the_mod) => {
                        if force {
                            pool.add_to_remotes_unchecked(&the_mod);
                        } else {
                            pool.add_to_remotes_checked(&the_mod)?;
                        }

                        if !cli.quiet {
                            print!("{}", the_mod.display_with_options(display_options));
                        }
                    }
                    Err(e) => eprintln!("Error adding remote mod: {e}"),
                }
            }

            pool.save().context("Saving the pool")?;
        }
        Commands::List => {
            let pool = Pool::new(&cli.pool_dir).context("Error initializing the pool")?;

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
        Commands::Remove { names } => {
            let mut pool = Pool::new(&cli.pool_dir).context("Error initializing the pool")?;

            for name in names {
                if !pool.remove_mod(&name)? && !cli.quiet {
                    println!("No mod {} was removed", name.italic().blue());
                }
            }

            pool.save().context("Saving the pool")?;
        }
        Commands::Info {
            display_options,
            target,
        } => {
            let mods: anyhow::Result<Vec<SearchedMod>> = match target {
                SearchTargets::Id { mod_ids } => mod_ids
                    .into_iter()
                    .map(|id| searcher.search_mod_by_id(id))
                    .collect(),
                SearchTargets::Slug { mod_slugs } => mod_slugs
                    .into_iter()
                    .map(|slug| searcher.search_mod_by_slug(&slug))
                    .collect(),
                SearchTargets::Jar { paths } => {
                    for path in paths {
                        let jar = JarMod::open(path)?;

                        println!("Name: {}", jar.name().blue().italic());
                        println!("Version: {}", jar.version());
                        println!(
                            "Minecraft version required: {}",
                            jar.minecraft_version().unwrap_or("Any")
                        );
                        println!(
                            "Loader version required: {}",
                            jar.loader_version().unwrap_or("Any")
                        );

                        let dependencies = jar.dependencies();

                        if !dependencies.is_empty() {
                            println!();
                            println!("Dependencies:");

                            for (name, version) in dependencies {
                                println!("\t- {name}({version})", name = name.green().italic());
                            }
                        }

                        let incompatibilities = jar.incompatibilities();

                        if !incompatibilities.is_empty() {
                            println!();
                            println!("Incompatibilities:");

                            for (name, version) in incompatibilities {
                                println!("\t- {name}({version})", name = name.red().bold());
                            }
                        }
                    }

                    return Ok(());
                }
            };

            for the_mod in mods? {
                println!("{}", the_mod.display_with_options(display_options));
            }
        }
        Commands::Tree => {
            searcher.set_silent(true); // Make it silent
            let pool = Pool::new(&cli.pool_dir).context("Error initializing the pool")?;

            let mut tree = TreeBuilder::new(String::from("Tmod"));

            fn add_remote_to_tree(
                searcher: &Searcher,
                tree: &mut TreeBuilder,
                the_mod: &SearchedMod,
                config: &Config,
            ) -> anyhow::Result<()> {
                tree.begin_child(the_mod.slug().to_string());

                let files = searcher.get_mod_files(the_mod, config)?;
                let file = files.first().with_context(|| {
                    format!("No files fetched for the mod '{}'", the_mod.slug())
                })?;

                for dep in file.dependencies() {
                    add_remote_to_tree(
                        searcher,
                        tree,
                        &searcher.search_mod_by_id(dep.id())?,
                        config,
                    )?;
                }

                tree.end_child();

                Ok(())
            }

            fn add_local_to_tree(
                searcher: &Searcher,
                tree: &mut TreeBuilder,
                the_mod: &JarMod,
                config: &Config,
            ) -> anyhow::Result<()> {
                tree.begin_child(the_mod.name().to_string());

                for dep in the_mod.dependencies().keys() {
                    if let Ok(remote) = searcher.search_mod_by_slug(dep) {
                        add_remote_to_tree(searcher, tree, &remote, config)?;
                    } else {
                        tree.add_empty_child(dep.to_string());
                    }
                }

                tree.end_child();

                Ok(())
            }

            tree.begin_child(String::from("Remotes"));

            for slug in pool.remotes() {
                let the_mod = searcher
                    .search_mod_by_slug(slug)
                    .expect("If remote mod is in the pool, it exists");

                add_remote_to_tree(&searcher, &mut tree, &the_mod, pool.config())?;
            }

            tree.end_child();
            tree.begin_child(String::from("Locals"));

            for local in pool.locals() {
                add_local_to_tree(&searcher, &mut tree, local, pool.config())?;
            }

            tree.end_child();

            ptree::print_tree(&tree.build()).context("Error displaying the tree")?;
        }
    }

    Ok(())
}
