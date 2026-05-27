pub mod adapters;
pub mod application;
pub mod cli;
pub mod config;
pub mod domain;
pub mod infrastructure;
pub mod utils;

use anyhow::Result;
use clap::Parser;

use crate::{adapters::App, cli::Cli};

pub async fn run() -> Result<()> {
    run_with_cli(Cli::parse()).await
}

pub async fn run_with_cli(cli: Cli) -> Result<()> {
    if cli.list_voices {
        cli::print_available_voices();
        return Ok(());
    }

    let config = config::ResolvedConfig::from_cli(&cli)?;
    utils::init_tracing(config.verbose);

    App::bootstrap(config)?.run().await
}
