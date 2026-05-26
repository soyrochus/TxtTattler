mod adapters;
mod application;
mod cli;
mod config;
mod domain;
mod infrastructure;
mod utils;

use anyhow::Result;
use clap::Parser;

use crate::{adapters::App, cli::Cli};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.list_voices {
        cli::print_available_voices();
        return Ok(());
    }

    let config = config::ResolvedConfig::from_cli(&cli)?;
    utils::init_tracing(config.verbose);

    let app = App::bootstrap(config)?;
    app.run().await
}
