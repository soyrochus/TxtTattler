use anyhow::Result;
use clap::Parser;
use console::style;
use txttattler::adapters::App;
use txttattler::cli::{Cli, VOICES};
use txttattler::config::AppConfig;
use txttattler::utils::{banner, init_tracing};

#[tokio::main]
async fn main() -> Result<()> {
    if dotenvy::dotenv().is_err() {
        dotenvy::from_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".env")).ok();
    }

    let cli = Cli::parse();
    init_tracing(cli.verbose);

    if cli.list_voices {
        println!("{}", banner());
        println!("{}", style("Available voices").cyan().bold());
        for voice in VOICES {
            println!("  - {voice}");
        }
        return Ok(());
    }

    println!("{}", banner());

    let config = AppConfig::load(cli.config.as_deref(), cli.azure)?;
    let app = App::new(config, cli.azure)?;
    let result = app.run(cli.into_request()).await?;

    let saved = result
        .saved_to
        .as_ref()
        .map(|path| format!(" Saved to {}", path.display()))
        .unwrap_or_default();
    let cache = result
        .cache_path
        .as_ref()
        .map(|path| {
            if result.cache_hit {
                format!(" Reused cache {}", path.display())
            } else {
                format!(" Cached at {}", path.display())
            }
        })
        .unwrap_or_default();

    println!(
        "{}",
        style(format!(
            "TxtTattler spilled the tea: {} chunk(s), {} bytes.{}{}",
            result.chunks, result.bytes, saved, cache
        ))
        .green()
        .bold()
    );

    Ok(())
}
