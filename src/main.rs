use clap::Parser;

mod cli;

use cli::Cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    Cli::get_searcher_mut().set_silent(cli.quiet);
    cli.run()
}
