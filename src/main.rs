use clap::Parser;

mod cli;

use cli::Cli;
use tmod::fetcher::SEARCHER;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    SEARCHER.set_silent(cli.quiet);

    cli.run(&mut std::io::stdout())
}
