use anyhow::{Context, Result, bail};
use clap::Parser;
use console::style;
use tracing_subscriber::{EnvFilter, fmt};
use txttattler::adapters::App;
use txttattler::cli::Cli;
use txttattler::config::Settings;
use txttattler::domain::entities::AVAILABLE_VOICES;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        fmt()
            .with_env_filter(
                EnvFilter::from_default_env().add_directive("txttattler=debug".parse()?),
            )
            .compact()
            .init();
    } else {
        fmt()
            .with_env_filter(
                EnvFilter::from_default_env().add_directive("txttattler=warn".parse()?),
            )
            .without_time()
            .with_target(false)
            .compact()
            .init();
    }

    if cli.list_voices {
        println!("{}", style("TxtTattler voices 🗣️").bold().cyan());
        for voice in AVAILABLE_VOICES {
            println!("  - {voice}");
        }
        return Ok(());
    }

    let file = cli
        .file
        .clone()
        .context("missing input file; try `txttattler --help`")?;

    if cli.no_play && cli.no_cache && cli.output.is_none() {
        bail!(
            "--no-play with --no-cache needs --output, otherwise the tattled MP3 has nowhere to go"
        );
    }

    let settings = Settings::load(cli.config.as_deref(), &cli)?;
    let app = App::new(settings, cli.clone())?;
    let report = app.run(&file).await?;

    println!(
        "{}",
        style(format!(
            "Done. The tattler spilled {} chars across {} chunk(s).",
            report.character_count, report.chunk_count
        ))
        .green()
        .bold()
    );

    if let Some(path) = report.output_path {
        println!("Saved: {}", style(path.display()).cyan());
    }
    if let Some(path) = report.cache_path {
        println!("Cache: {}", style(path.display()).dim());
    }

    Ok(())
}
