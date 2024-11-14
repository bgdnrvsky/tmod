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
    fetcher::{mod_search::search_mod::display::ModOptions, SEARCHER},
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
        /// Mark mod as client only, it (and dependencies) will be ignored when installing with `--server`
        #[arg(short, long, default_value_t = false)]
        client_only: bool,
        #[command(subcommand)]
        add_target: ModTargets,
    },
    /// Remove a mod from the `pool`
    Remove {
        #[arg(required = true)]
        slugs: Vec<String>,
    },
    /// Search a remote mod and print its info
    Info {
        #[command(subcommand)]
        search_target: ModTargets,
        /// If not specified, fetches the latest available version of the mod
        #[arg(short, long)]
        timestamp: Option<chrono::DateTime<chrono::Utc>>,
        #[clap(flatten)]
        config: Option<Config>,
        #[clap(flatten)]
        mod_display_options: ModOptions,
    },
    /// Download all the mods to the folder
    Install {
        /// Do not install client only mods (and dependencies)
        #[arg(short, long, default_value_t = false)]
        server: bool,
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
}

impl Cli {
    pub fn run(&self, writer: &mut impl Write) -> anyhow::Result<()> {
        match &self.command {
            Commands::Init => Pool::init(&self.pool_dir).map(|_| ()),
            Commands::List => {
                let pool = self.read_pool()?;

                let remotes = pool.manually_added;

                if !remotes.is_empty() {
                    writeln!(writer, "Remotes:")?;
                    for r in remotes.iter() {
                        writeln!(writer, "\t- {}", r.italic().blue())?;
                    }
                } else {
                    return writeln!(writer, "Empty!").context("IO");
                }

                Ok(())
            }
            Commands::Add {
                add_target,
                client_only,
            } => {
                let mut pool = self.read_pool()?;

                let remote_mod = match add_target {
                    ModTargets::Id { mod_id } => SEARCHER.search_mod_by_id(*mod_id)?,
                    ModTargets::Slug { mod_slug } => SEARCHER.search_mod_by_slug(mod_slug)?,
                };

                pool.add_to_remotes(&remote_mod, *client_only, true)?;

                if !self.quiet {
                    write!(
                        writer,
                        "{}",
                        remote_mod.display_with_options(ModOptions::default())
                    )?;
                }

                self.save_pool(pool)
            }
            Commands::Remove { slugs: names } => {
                let mut pool = self.read_pool()?;

                for name in names {
                    if !pool.remove_mod(name) && !self.quiet {
                        writeln!(writer, "No mod {} was removed", name.italic().blue())?;
                    }
                }

                self.save_pool(pool)
            }
            Commands::Info {
                mod_display_options,
                search_target,
                timestamp,
                config,
            } => {
                let remote_mod = match search_target {
                    ModTargets::Id { mod_id } => SEARCHER.search_mod_by_id(*mod_id),
                    ModTargets::Slug { mod_slug } => SEARCHER.search_mod_by_slug(mod_slug),
                }?;

                writeln!(
                    writer,
                    "{}",
                    remote_mod.display_with_options(*mod_display_options)
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
                let pool = self.read_pool()?;

                let mut tree = TreeBuilder::new(String::from("Tmod"));

                fn add_recursive_to_tree(
                    slug: impl AsRef<str>,
                    tree: &mut TreeBuilder,
                    pool: &Pool,
                ) {
                    tree.begin_child(slug.as_ref().to_string());

                    for dep in pool
                        .locks
                        .get(slug.as_ref())
                        .expect("If this fails, the lock file is invalid")
                        .dependencies
                        .iter()
                    {
                        add_recursive_to_tree(dep, tree, pool);
                    }

                    tree.end_child();
                }

                for slug in pool.manually_added.iter() {
                    add_recursive_to_tree(slug, &mut tree, &pool);
                }

                ptree::write_tree(&tree.build(), writer).context("Displaying tree")
            }
            Commands::Install { out_dir, server } => {
                let pool = self.read_pool()?;

                // Create output directory
                DirBuilder::new()
                    .recursive(true)
                    .create(out_dir)
                    .with_context(|| {
                        format!("Creating output directory '{}'", out_dir.display())
                    })?;

                fn install_mod(
                    is_server: bool,
                    out_dir: &std::path::Path,
                    pool: &Pool,
                    slug: impl AsRef<str>,
                ) -> anyhow::Result<()> {
                    let dep_info = pool.locks.get(slug.as_ref()).context("Invalid lock file")?;

                    if dep_info.client_only && is_server {
                        // Ignore the mod
                        return Ok(());
                    }

                    let the_mod = SEARCHER.search_mod_by_slug(slug)?;
                    let file = SEARCHER.get_specific_mod_file(
                        &the_mod,
                        &pool.config,
                        Some(dep_info.timestamp),
                    )?;

                    if !out_dir
                        .join(&file.file_name)
                        .try_exists()
                        .is_ok_and(|exists| exists)
                    {
                        // Download the file
                        let response = SEARCHER.download_file(&file)?;

                        // Create the file
                        let path = &out_dir.join(file.file_name);
                        let mut file = File::create(path)
                            .with_context(|| format!("Creating file '{}'", path.display()))?;

                        std::io::copy(&mut response.into_reader(), &mut file).with_context(
                            || format!("Writing content to the file '{}'", path.display()),
                        )?;
                    }

                    for slug in dep_info.dependencies.iter() {
                        install_mod(is_server, out_dir, pool, slug)?;
                    }

                    Ok(())
                }

                for slug in pool.manually_added.iter() {
                    install_mod(*server, out_dir, &pool, slug)?;
                }

                Ok(())
            }
        }
    }

    fn read_pool(&self) -> anyhow::Result<Pool> {
        Pool::read(&self.pool_dir).with_context(|| {
            format!(
                "Error initializing the pool from `{}`",
                self.pool_dir.display()
            )
        })
    }

    fn save_pool(&self, pool: Pool) -> anyhow::Result<()> {
        pool.save()
            .with_context(|| format!("Saving the pool to `{}`", self.pool_dir.display()))
    }
}
