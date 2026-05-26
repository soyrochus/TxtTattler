//! TxtTattler — the Rust CLI that tattles on your text files.
//!
//! Entry point. All the serious work happens in the domain + application layers.
//! This file is mostly "parse args, load config, wire adapters, run use-case, be dramatic."

use anyhow::Result;
use clap::Parser;

mod adapters;
mod application;
mod cli;
mod config;
mod domain;
mod infrastructure;
mod utils;

use crate::application::text_to_speech::TextToSpeechOrchestrator;
use crate::domain::ports::TextToSpeechService;  // needed to bring .tattle() into scope
use crate::cli::Cli;
use crate::config::{load_config, print_config_table, ResolvedConfig};

use crate::infrastructure::audio::RodioAudioPlayer;
use crate::infrastructure::tts::OpenAiTtsProvider;
use crate::utils::{print_banner, print_success_footer};

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env from the current working directory (CWD) if present.
    // This is the standard developer workflow: keep OPENAI_API_KEY (and
    // AZURE_* vars) in a local .env file instead of exporting them globally.
    let env_loaded = dotenvy::dotenv().ok();

    let cli = Cli::parse();

    // Early exit for --list-voices (before we complain about missing keys)
    if cli.list_voices {
        cli::print_voice_list();
        return Ok(());
    }

    // Setup logging based on -V count
    setup_tracing(cli.verbose);

    // Report .env loading at verbose level (now that tracing is live)
    if cli.verbose > 0 {
        match env_loaded {
            Some(path) => tracing::info!("Loaded environment variables from {}", path.display()),
            None => tracing::debug!("No .env file found in current directory (this is fine)"),
        }
    }

    // Load optional config file
    let config_path = cli.config.clone().unwrap_or_else(config::default_config_path);
    let file_cfg = load_config(cli.config.as_deref())?;

    // Resolve final settings (CLI > env > file)
    let resolved = ResolvedConfig::from_sources(&cli, &file_cfg)?;

    if cli.verbose > 0 {
        print_banner();
        print_config_table(&resolved, &config_path);
    } else {
        print_banner();
    }

    // Handle subcommands
    if let Some(crate::cli::Commands::Config { toml }) = &cli.command {
        if *toml {
            println!("{}", toml::to_string_pretty(&file_cfg)?);
        } else {
            print_config_table(&resolved, &config_path);
        }
        return Ok(());
    }

    // We need a file at this point
    let file_path = cli.file.clone().expect("FILE is required");

    // Wire the adapters (dependency injection — the "hexagonal" part)
    let reader_registry = &*adapters::FILE_READER_REGISTRY;

    // Create TTS provider (OpenAI or Azure)
    let tts_provider: Box<dyn crate::domain::ports::TtsProvider> = if resolved.is_azure_mode() {
        // Try explicit config values first, fall back to env
        match OpenAiTtsProvider::from_config(
            resolved.azure_api_key.clone(),
            resolved.azure_endpoint.clone(),
            true,
        ) {
            Ok(p) => Box::new(p),
            Err(_) => Box::new(OpenAiTtsProvider::from_env(true)?),
        }
    } else {
        match OpenAiTtsProvider::from_config(
            resolved.openai_api_key.clone(),
            resolved.openai_base_url.clone(),
            false,
        ) {
            Ok(p) => Box::new(p),
            Err(_) => Box::new(OpenAiTtsProvider::from_env(false)?),
        }
    };

    let audio_player: Box<dyn crate::domain::ports::AudioPlayer> = Box::new(RodioAudioPlayer);

    // Build the orchestrator (the use-case)
    let orchestrator = TextToSpeechOrchestrator::new(reader_registry, tts_provider, audio_player);

    // Load the document (the only place we care about file format)
    let document = orchestrator.load_document(&file_path)?;

    // Decide playback behavior
    let should_play = !cli.no_play;

    // Run the main gossip delivery (with shiny new MP3 caching)
    let result = orchestrator
        .tattle(
            document,
            resolved.tts_options,
            cli.output.clone(),
            should_play,
            cli.no_cache,
            cli.refresh,
            cli.cache_dir.clone(),
        )
        .await?;

    print_success_footer(&result);

    Ok(())
}

fn setup_tracing(verbosity: u8) {
    let level = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    // Allow user to override completely with RUST_LOG
    let env_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| {
        format!("txttattler={},async_openai=warn,rodio=warn", level)
    });

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_line_number(verbosity > 2)
        .compact()
        .init();
}
