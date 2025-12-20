mod cli;
mod index;
mod output;
mod search;
mod walker;



use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Search(args) => {
            search::run_search(args)?;
        }
        Commands::Init(args) => {
            index::init_index(args)?;
        }
    }

    Ok(())
}