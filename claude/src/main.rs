mod adapters;
mod application;
mod cli;
mod config;
mod domain;
mod infrastructure;
mod utils;

use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use console::style;
use tracing_subscriber::EnvFilter;

use crate::adapters::FileReaderRegistry;
use crate::application::text_to_speech::{TextToSpeechArgs, TextToSpeechUseCase};
use crate::cli::Cli;
use crate::config::Config;
use crate::domain::entities::{TtsModel, TtsOptions, Voice};
use crate::infrastructure::audio::player::RodioPlayer;
use crate::infrastructure::tts::openai::OpenAiTtsProvider;
use crate::utils::DefaultTextProcessor;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Init tracing
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("warn")
    };
    tracing_subscriber::fmt().with_env_filter(filter).init();

    // Load .env (ignore if not found)
    let _ = dotenvy::dotenv();

    // Handle --list-voices
    if cli.list_voices {
        println!("{}", style("Available voices:").bold());
        for v in Voice::all() {
            println!("  - {}", style(v).cyan());
        }
        return Ok(());
    }

    // Require input file
    let input_path = cli.file.ok_or_else(|| {
        anyhow::anyhow!(
            "No input file specified. Usage: txttattler <file.txt>\nRun with --help for more options."
        )
    })?;

    if !input_path.exists() {
        return Err(anyhow::anyhow!(
            "Input file '{}' does not exist.",
            input_path.display()
        ));
    }

    // Validate speed
    if !(0.25..=4.0).contains(&cli.speed) {
        return Err(anyhow::anyhow!(
            "Speed must be between 0.25 and 4.0 (got {})",
            cli.speed
        ));
    }

    // Load config
    let cfg = Config::load(cli.config.as_deref())
        .context("Failed to load configuration")?;

    // Resolve voice: CLI > config > default
    let voice_str = if cli.voice != "alloy" {
        cli.voice.clone()
    } else {
        cfg.voice.unwrap_or_else(|| cli.voice.clone())
    };
    let voice = Voice::from_str(&voice_str)?;

    // Resolve model: CLI > config > default
    let model_str = if cli.model != "tts-1" {
        cli.model.clone()
    } else {
        cfg.model.unwrap_or_else(|| cli.model.clone())
    };
    let model = TtsModel::from_str(&model_str)?;

    // Resolve speed: CLI > config > default
    let speed = if (cli.speed - 1.0f32).abs() > f32::EPSILON {
        cli.speed
    } else {
        cfg.speed.unwrap_or(cli.speed)
    };

    let options = TtsOptions { voice, model, speed };

    // Build TTS provider
    let use_azure = cli.azure || std::env::var("AZURE_OPENAI_ENDPOINT").is_ok();
    let tts_provider: Arc<dyn crate::domain::ports::TtsProvider> = if use_azure {
        let endpoint = std::env::var("AZURE_OPENAI_ENDPOINT")
            .or_else(|_| cfg.azure.endpoint.ok_or_else(|| std::env::VarError::NotPresent))
            .context("AZURE_OPENAI_ENDPOINT not set (required for Azure mode)")?;
        let api_key = std::env::var("AZURE_OPENAI_API_KEY")
            .or_else(|_| cfg.azure.api_key.ok_or_else(|| std::env::VarError::NotPresent))
            .context("AZURE_OPENAI_API_KEY not set (required for Azure mode)")?;
        let deployment = std::env::var("AZURE_OPENAI_DEPLOYMENT_ID")
            .or_else(|_| cfg.azure.deployment_id.ok_or_else(|| std::env::VarError::NotPresent))
            .unwrap_or_else(|_| "tts".to_string());
        Arc::new(OpenAiTtsProvider::new_azure(&endpoint, &api_key, &deployment))
    } else {
        Arc::new(OpenAiTtsProvider::new_standard())
    };

    // Build file reader registry and pick a reader for the input
    let registry = FileReaderRegistry::new();
    let reader = registry.get_for_path(&input_path)?;

    // Build audio player
    let player: Arc<dyn crate::domain::ports::AudioPlayer> = Arc::new(RodioPlayer);

    // Build text processor
    let processor: Arc<dyn crate::domain::ports::TextProcessor> = Arc::new(DefaultTextProcessor);

    // Build use case
    let use_case = TextToSpeechUseCase::new(reader, tts_provider, player, processor);

    // Build args
    let args = TextToSpeechArgs {
        input_path,
        output_path: cli.output,
        cache_dir: cli.cache_dir.or(cfg.cache_dir),
        options,
        no_play: cli.no_play,
        no_cache: cli.no_cache,
        refresh: cli.refresh,
        verbose: cli.verbose,
    };

    use_case.run(args).await
}
