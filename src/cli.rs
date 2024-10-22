use std::{
    io::Write,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

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
pub struct Cli {
    #[arg(long, default_value = ".tmod", value_name = "PATH")]
    pool_dir: PathBuf,
    #[command(subcommand)]
    command: Commands,
    /// Dot not print Tmod log messages
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,
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
        add_target: SearchTargets,
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
        #[command(subcommand)]
        target: SearchTargets,
        /// If not specified, fetches the latest available version of the mod
        #[arg(short, long)]
        timestamp: Option<chrono::DateTime<chrono::Utc>>,
        #[clap(flatten)]
        config: Option<Config>,
        #[clap(flatten)]
        display_options: ModOptions,
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

impl Cli {
    pub fn run(&self, writer: &mut impl Write) -> anyhow::Result<()> {
        match &self.command {
            Commands::Init => Pool::init(&self.pool_dir).map(|_| ()),
            Commands::List => {
                let pool = self.new_pool()?;

                let remotes = pool.manually_added;
                let locals = pool.locals;

                if remotes.is_empty() && locals.is_empty() {
                    writeln!(writer, "Empty!")?;
                    return Ok(());
                }

                if !remotes.is_empty() {
                    writeln!(writer, "Remotes:")?;
                    for r in remotes.iter() {
                        writeln!(writer, "\t- {}", r.italic().blue())?;
                    }
                }

                if !locals.is_empty() {
                    writeln!(writer, "Locals:")?;
                    for l in locals {
                        writeln!(writer, "\t- {}", l.name().italic().blue())?;
                    }
                }

                Ok(())
            }
            Commands::Add {
                force,
                display_options,
                add_target,
                r#move,
            } => {
                let searcher = Self::get_searcher();
                let mut pool = self.new_pool()?;

                let mods: Vec<anyhow::Result<_>> = match add_target {
                    SearchTargets::Id { mod_ids } => mod_ids
                        .iter()
                        .map(|&id| searcher.search_mod_by_id(id))
                        .collect(),
                    SearchTargets::Slug { mod_slugs } => mod_slugs
                        .iter()
                        .map(|slug| searcher.search_mod_by_slug(slug))
                        .collect(),
                    SearchTargets::Jar { paths } => {
                        for path in paths {
                            match JarMod::open(path) {
                                Ok(jar) => {
                                    let to =
                                        pool.locals_path().join(jar.name()).with_extension("jar");

                                    if *r#move {
                                        std::fs::rename(path, to).context("Moving jar")?;

                                        if !self.quiet {
                                            writeln!(writer, "Moving {}", path.display())?;
                                        }
                                    } else if !self.quiet {
                                        std::fs::copy(path, to).context("Copying jar")?;

                                        writeln!(writer, "Copying {}", path.display())?;
                                    }

                                    pool.add_to_locals(jar);
                                }
                                Err(e) => eprintln!("Error adding local mod: {e}"),
                            }
                        }

                        return Ok(());
                    }
                };

                drop(searcher);

                for the_mod in mods {
                    match the_mod {
                        Ok(the_mod) => {
                            let file = Self::get_searcher().get_specific_mod_file(
                                &the_mod,
                                &pool.config,
                                None,
                            )?;

                            if *force {
                                pool.add_to_remotes_unchecked(&the_mod, file, true)?;
                            } else {
                                pool.add_to_remotes_checked(&the_mod, file, true)?;
                            }

                            if !self.quiet {
                                write!(
                                    writer,
                                    "{}",
                                    the_mod.display_with_options(*display_options)
                                )?;
                            }
                        }
                        Err(e) => eprintln!("Couldn't add remote mod: {}", e),
                    }
                }

                pool.save().context("Saving the pool")
            }
            Commands::Remove { names } => {
                let mut pool = self.new_pool()?;

                for name in names {
                    if !pool.remove_mod(name)? && !self.quiet {
                        writeln!(writer, "No mod {} was removed", name.italic().blue())?;
                    }
                }

                pool.save().context("Saving the pool")
            }
            Commands::Info {
                display_options,
                target,
                timestamp,
                config,
            } => {
                let searcher = Self::get_searcher();

                let mods: Vec<anyhow::Result<_>> = match target {
                    SearchTargets::Id { mod_ids } => mod_ids
                        .iter()
                        .map(|&id| searcher.search_mod_by_id(id))
                        .collect(),
                    SearchTargets::Slug { mod_slugs } => mod_slugs
                        .iter()
                        .map(|slug| searcher.search_mod_by_slug(slug))
                        .collect(),
                    SearchTargets::Jar { paths } => {
                        for path in paths {
                            let jar = JarMod::open(path)?;

                            writeln!(writer, "Name: {}", jar.name().blue().italic())?;
                            writeln!(writer, "Version: {}", jar.version())?;
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
                                writeln!(writer)?;
                                writeln!(writer, "Dependencies:")?;

                                for (name, version) in dependencies {
                                    writeln!(
                                        writer,
                                        "\t- {name}({version})",
                                        name = name.green().italic()
                                    )?;
                                }
                            }

                            let incompatibilities = jar.incompatibilities();

                            if !incompatibilities.is_empty() {
                                writeln!(writer)?;
                                writeln!(writer, "Incompatibilities:")?;

                                for (name, version) in incompatibilities {
                                    writeln!(
                                        writer,
                                        "\t- {name}({version})",
                                        name = name.red().bold()
                                    )?;
                                }
                            }
                        }

                        return Ok(());
                    }
                };

                for the_mod in mods {
                    match the_mod {
                        Ok(the_mod) => {
                            let file = match config {
                                Some(config) => Some(
                                    searcher.get_specific_mod_file(&the_mod, config, *timestamp)?,
                                ),
                                None => None,
                            };

                            writeln!(writer, "{}", the_mod.display_with_options(*display_options))?;

                            if let Some(file) = file {
                                let relations = file.relations;

                                if !relations.is_empty() {
                                    writeln!(writer, "Relations:")?;
                                }

                                for relation in relations.iter() {
                                    println!(
                                        "\t - {id} ({rel_type:?})",
                                        id = relation.id,
                                        rel_type = relation.relation
                                    );
                                }
                            }
                        }
                        Err(e) => eprintln!("Couldn't get info for the mod: {e}"),
                    }
                }

                Ok(())
            }
            Commands::Tree => {
                let mut searcher = Self::get_searcher_mut();
                searcher.set_silent(true); // Make it silent

                let pool = self.new_pool()?;

                let mut tree = TreeBuilder::new(String::from("Tmod"));

                fn add_remote_to_tree(
                    searcher: &Searcher,
                    tree: &mut TreeBuilder,
                    the_mod: &SearchedMod,
                    config: &Config,
                ) -> anyhow::Result<()> {
                    tree.begin_child(the_mod.slug.to_string());

                    let files = searcher.get_mod_files(the_mod, config)?;
                    let file = files.iter().max_by_key(|file| file.date).with_context(|| {
                        format!("No files fetched for the mod '{}'", the_mod.slug)
                    })?;

                    for dep in file.relations.iter() {
                        add_remote_to_tree(
                            searcher,
                            tree,
                            &searcher.search_mod_by_id(dep.id)?,
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

                for slug in pool.manually_added.iter() {
                    let the_mod = searcher
                        .search_mod_by_slug(slug)
                        .expect("If remote mod is in the pool, it exists");

                    add_remote_to_tree(&searcher, &mut tree, &the_mod, &pool.config)?;
                }

                tree.end_child();
                tree.begin_child(String::from("Locals"));

                for local in pool.locals.iter() {
                    add_local_to_tree(&searcher, &mut tree, local, &pool.config)?;
                }

                tree.end_child();

                ptree::print_tree(&tree.build()).context("Error displaying the tree")
            }
        }
    }

    pub fn get_searcher() -> impl Deref<Target = Searcher> {
        SEARCHER
            .try_lock()
            .expect("Should only be one single searcher user at a time")
    }

    pub fn get_searcher_mut() -> impl DerefMut<Target = Searcher> {
        SEARCHER
            .try_lock()
            .expect("Should only be one single searcher user at a time")
    }

    fn new_pool(&self) -> anyhow::Result<Pool> {
        Pool::read(&self.pool_dir).with_context(|| {
            format!(
                "Error initializing the pool from {}",
                self.pool_dir.display()
            )
        })
    }
}
