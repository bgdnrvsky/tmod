use clap::Parser;

mod cli;

use cli::Cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    {
        let mut searcher = Cli::get_searcher_mut();
        searcher.set_silent(cli.quiet);
    }

    cli.run()
}
