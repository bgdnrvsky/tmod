use std::{
    fs::{DirBuilder, File},
    io::Write,
    path::PathBuf,
};

use anyhow::Context;
use clap::{Parser, Subcommand};
use colored::Colorize;
use ptree::TreeBuilder;
use tmod::{
    fetcher::{
        mod_search::search_mod::{display::ModOptions, SearchedMod},
        SEARCHER,
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
        #[command(subcommand)]
        add_target: ModTargets,
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
        target: ModTargets,
        /// If not specified, fetches the latest available version of the mod
        #[arg(short, long)]
        timestamp: Option<chrono::DateTime<chrono::Utc>>,
        #[clap(flatten)]
        config: Option<Config>,
        #[clap(flatten)]
        display_options: ModOptions,
    },
    /// Download all the mods to the folder
    Install {
        #[arg(short, long, default_value = "mods", value_name = "PATH")]
        out_dir: PathBuf,
    },
    Tree,
}

#[derive(Debug, Subcommand)]
enum ModTargets {
    /// Using CurseForge mod id
    Id { mod_id: usize },
    /// Using mod's 'slug' (slug is not always the same as the mod name)
    Slug { mod_slug: String },
    /// Using your local Jar file
    Jar { path: PathBuf },
}

impl Cli {
    pub fn run(&self, writer: &mut impl Write) -> anyhow::Result<()> {
        match &self.command {
            Commands::Init => Pool::init(&self.pool_dir).map(|_| ()),
            Commands::List => {
                let pool = self.read_pool()?;

                let remotes = pool.manually_added;
                let locals = pool.locals;

                if remotes.is_empty() && locals.is_empty() {
                    return writeln!(writer, "Empty!").context("IO");
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
            Commands::Add { add_target, r#move } => {
                let mut pool = self.read_pool()?;

                let remote_mod = match add_target {
                    ModTargets::Jar { path } => {
                        let jar = JarMod::open(path)
                            .with_context(|| format!("Opening jar '{}'", path.display()))?;

                        let to = pool.locals_path().join(jar.name()).with_extension("jar");

                        if *r#move {
                            let moving_context = || format!("Moving {}", path.display());
                            std::fs::rename(path, to).with_context(moving_context)?;

                            if !self.quiet {
                                writeln!(writer, "{}", moving_context())?;
                            }
                        } else {
                            let copying_context = || format!("Copying {}", path.display());
                            std::fs::copy(path, to).with_context(copying_context)?;

                            if !self.quiet {
                                writeln!(writer, "{}", copying_context())?;
                            }
                        }

                        pool.add_to_locals(jar).context("Adding jar to the pool")?;
                        return pool.save().context("Saving the pool");
                    }
                    ModTargets::Id { mod_id } => SEARCHER.search_mod_by_id(*mod_id)?,
                    ModTargets::Slug { mod_slug } => SEARCHER.search_mod_by_slug(mod_slug)?,
                };

                pool.add_to_remotes(&remote_mod, true)?;

                if !self.quiet {
                    write!(
                        writer,
                        "{}",
                        remote_mod.display_with_options(ModOptions::default())
                    )?;
                }

                pool.save().context("Saving the pool")
            }
            Commands::Remove { names } => {
                let mut pool = self.read_pool()?;

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
                let remote_mod = match target {
                    ModTargets::Id { mod_id } => SEARCHER.search_mod_by_id(*mod_id),
                    ModTargets::Slug { mod_slug } => SEARCHER.search_mod_by_slug(mod_slug),
                    ModTargets::Jar { path } => {
                        let jar = JarMod::open(path)?;

                        writeln!(writer, "Name: {}", jar.name().blue().italic())?;
                        writeln!(writer, "Version: {}", jar.version())?;
                        writeln!(
                            writer,
                            "Minecraft version required: {}",
                            jar.minecraft_version().unwrap_or("Any")
                        )?;
                        writeln!(
                            writer,
                            "Loader version required: {}",
                            jar.loader_version().unwrap_or("Any")
                        )?;

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

                        return Ok(());
                    }
                }?;

                writeln!(
                    writer,
                    "{}",
                    remote_mod.display_with_options(*display_options)
                )?;

                if let Some(config) = config {
                    let relations = SEARCHER
                        .get_specific_mod_file(&remote_mod, config, *timestamp)?
                        .relations;

                    if !relations.is_empty() {
                        writeln!(writer, "Relations:")?;
                    }

                    for relation in relations.iter() {
                        writeln!(
                            writer,
                            "\t - {id} ({rel_type:?})",
                            id = relation.id,
                            rel_type = relation.relation
                        )?;
                    }
                }

                Ok(())
            }
            Commands::Tree => {
                SEARCHER.set_silent(true); // Make it silent

                let pool = self.read_pool()?;

                let mut tree = TreeBuilder::new(String::from("Tmod"));

                fn add_remote_to_tree(
                    tree: &mut TreeBuilder,
                    the_mod: &SearchedMod,
                    config: &Config,
                ) -> anyhow::Result<()> {
                    tree.begin_child(the_mod.slug.to_string());

                    let files = SEARCHER.get_mod_files(the_mod, config)?;
                    let file = files.iter().max_by_key(|file| file.date).with_context(|| {
                        format!("No files fetched for the mod '{}'", the_mod.slug)
                    })?;

                    for dep in file.relations.iter() {
                        add_remote_to_tree(tree, &SEARCHER.search_mod_by_id(dep.id)?, config)?;
                    }

                    tree.end_child();

                    Ok(())
                }

                fn add_local_to_tree(
                    tree: &mut TreeBuilder,
                    the_mod: &JarMod,
                    config: &Config,
                ) -> anyhow::Result<()> {
                    tree.begin_child(the_mod.name().to_string());

                    for dep in the_mod.dependencies().keys() {
                        if let Ok(remote) = SEARCHER.search_mod_by_slug(dep) {
                            add_remote_to_tree(tree, &remote, config)?;
                        } else {
                            tree.add_empty_child(dep.to_string());
                        }
                    }

                    tree.end_child();

                    Ok(())
                }

                tree.begin_child(String::from("Remotes"));

                for slug in pool.manually_added.iter() {
                    let the_mod = SEARCHER
                        .search_mod_by_slug(slug)
                        .expect("If remote mod is in the pool, it exists");

                    add_remote_to_tree(&mut tree, &the_mod, &pool.config)?;
                }

                tree.end_child();
                tree.begin_child(String::from("Locals"));

                for local in pool.locals.iter() {
                    add_local_to_tree(&mut tree, local, &pool.config)?;
                }

                tree.end_child();

                ptree::print_tree(&tree.build()).context("Error displaying the tree")
            }
            Commands::Install { out_dir } => {
                let pool = self.read_pool()?;

                // Create output directory
                DirBuilder::new().create(out_dir).with_context(|| {
                    format!("Creating output directory '{}'", out_dir.display())
                })?;

                fn install_mod(
                    out_dir: &std::path::Path,
                    pool: &Pool,
                    slug: impl AsRef<str>,
                ) -> anyhow::Result<()> {
                    let dep_info = pool.locks.get(slug.as_ref()).context("Invalid lock file")?;
                    let the_mod = SEARCHER.search_mod_by_slug(slug)?;
                    let file = SEARCHER.get_specific_mod_file(
                        &the_mod,
                        &pool.config,
                        Some(dep_info.timestamp),
                    )?;

                    // Download the file
                    let response = SEARCHER.download_file(&file)?;

                    // Create the file
                    let path = &out_dir.join(file.file_name);
                    let mut file = File::create(path)
                        .with_context(|| format!("Creating file '{}'", path.display()))?;

                    std::io::copy(&mut response.into_reader(), &mut file).with_context(|| {
                        format!("Writing content to the file '{}'", path.display())
                    })?;

                    for slug in dep_info.dependencies.iter() {
                        install_mod(out_dir, pool, slug)?;
                    }

                    Ok(())
                }

                for slug in pool.manually_added.iter() {
                    install_mod(out_dir, &pool, slug)?;
                }

                // Install local mods
                for local in pool.locals.iter() {
                    let file_name = local.path();
                    std::fs::copy(pool.locals_path().join(file_name), out_dir.join(file_name))?;

                    for dep in local.dependencies().keys() {
                        install_mod(out_dir, &pool, dep)?;
                    }
                }

                Ok(())
            }
        }
    }

    fn read_pool(&self) -> anyhow::Result<Pool> {
        Pool::read(&self.pool_dir).with_context(|| {
            format!(
                "Error initializing the pool from {}",
                self.pool_dir.display()
            )
        })
    }
}
