//! TxtTattler library crate.
//!
//! Architecture: clean hexagonal / ports & adapters.
//!
//! ```text
//!   cli ─┐
//!        ├─▶ application::TextToSpeech ──▶ domain::ports ◀── infrastructure adapters
//!   config ┘        (use-case)              (traits)        (txt reader, OpenAI, rodio)
//! ```
//!
//! The use-case depends only on the traits in [`domain::ports`]; every concrete
//! detail (file formats, OpenAI, audio backend) is a swappable adapter. Adding a
//! new file format means writing one [`domain::ports::FileReader`] and
//! registering it in [`adapters::FileReaderRegistry`] — nothing else changes.

pub mod adapters;
pub mod application;
pub mod cli;
pub mod config;
pub mod domain;
pub mod infrastructure;
pub mod utils;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::Context;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use tracing_subscriber::EnvFilter;

use crate::adapters::FileReaderRegistry;
use crate::application::text_to_speech::{
    ProgressReporter, TextToSpeech, TtsOutcome, TtsRequest,
};
use crate::cli::Cli;
use crate::config::Config;
use crate::domain::entities::{TtsModel, TtsOptions, Voice};
use crate::domain::ports::TtsProvider;
use crate::infrastructure::audio::player::RodioPlayer;
use crate::infrastructure::tts::openai::OpenAiTtsProvider;
use crate::utils::DefaultTextProcessor;

/// The full CLI flow: parse-driven configuration → run the use-case.
pub async fn run(cli: Cli) -> anyhow::Result<()> {
    init_tracing(cli.verbose);

    // Load `.env` from the working directory first, so it can populate the env
    // vars (OPENAI_API_KEY, AZURE_*) that everything below reads.
    let _ = dotenvy::dotenv();

    if cli.list_voices {
        print_voices();
        return Ok(());
    }

    let input_path = cli.file.clone().ok_or_else(|| {
        anyhow::anyhow!("no input file given. Try: txttattler myfile.txt   (or run --help)")
    })?;
    if !input_path.exists() {
        anyhow::bail!("file '{}' does not exist", input_path.display());
    }

    let cfg = Config::load(cli.config.as_deref()).context("failed to load configuration")?;

    // Precedence everywhere: CLI flag > config file > built-in default.
    let options = TtsOptions {
        voice: resolve_voice(&cli, &cfg)?,
        model: resolve_model(&cli, &cfg)?,
        speed: resolve_speed(&cli, &cfg),
    };
    options.validate()?;

    let cache_dir = cli
        .cache_dir
        .clone()
        .or_else(|| cfg.cache_dir.clone())
        .unwrap_or_else(default_cache_dir);

    let use_azure = azure_requested(&cli, &cfg);
    let provider = build_provider(use_azure, &cfg)?;

    let registry = FileReaderRegistry::with_builtins();
    let reader = registry.reader_for(&input_path)?;

    print_summary(&options, use_azure, &cache_dir, cli.no_cache);

    let use_case = TextToSpeech::new(
        reader,
        Arc::new(DefaultTextProcessor),
        provider,
        Arc::new(RodioPlayer),
    );

    let request = TtsRequest {
        input_path,
        output_path: cli.output.clone(),
        cache_dir,
        options,
        play: !cli.no_play,
        use_cache: !cli.no_cache,
        refresh: cli.refresh,
    };

    let reporter = ConsoleReporter::new(cli.verbose);
    use_case.run(request, &reporter).await?;
    Ok(())
}

fn init_tracing(verbose: bool) {
    let filter = if verbose {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"))
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"))
    };
    // try_init so repeated calls (e.g. in tests) don't panic.
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}

fn resolve_voice(cli: &Cli, cfg: &Config) -> anyhow::Result<Voice> {
    // If the user changed the flag away from its default, trust the flag;
    // otherwise let the config file have a say.
    let raw = if cli.voice != "alloy" {
        cli.voice.clone()
    } else {
        cfg.voice.clone().unwrap_or_else(|| cli.voice.clone())
    };
    raw.parse()
}

fn resolve_model(cli: &Cli, cfg: &Config) -> anyhow::Result<TtsModel> {
    let raw = if cli.model != "tts-1" {
        cli.model.clone()
    } else {
        cfg.model.clone().unwrap_or_else(|| cli.model.clone())
    };
    raw.parse()
}

fn resolve_speed(cli: &Cli, cfg: &Config) -> f32 {
    if (cli.speed - 1.0).abs() > f32::EPSILON {
        cli.speed
    } else {
        cfg.speed.unwrap_or(cli.speed)
    }
}

fn azure_requested(cli: &Cli, cfg: &Config) -> bool {
    cli.azure || std::env::var("AZURE_OPENAI_ENDPOINT").is_ok() || cfg.azure.endpoint.is_some()
}

fn build_provider(use_azure: bool, cfg: &Config) -> anyhow::Result<Arc<dyn TtsProvider>> {
    if use_azure {
        let endpoint = std::env::var("AZURE_OPENAI_ENDPOINT")
            .ok()
            .or_else(|| cfg.azure.endpoint.clone())
            .context("Azure mode needs AZURE_OPENAI_ENDPOINT (env, .env, or config)")?;
        let api_key = std::env::var("AZURE_OPENAI_API_KEY")
            .ok()
            .or_else(|| cfg.azure.api_key.clone())
            .context("Azure mode needs AZURE_OPENAI_API_KEY (env, .env, or config)")?;
        let deployment = std::env::var("AZURE_OPENAI_DEPLOYMENT_ID")
            .ok()
            .or_else(|| cfg.azure.deployment_id.clone())
            .unwrap_or_else(|| "tts".to_string());
        let api_version = std::env::var("AZURE_OPENAI_API_VERSION")
            .ok()
            .or_else(|| cfg.azure.api_version.clone());
        Ok(Arc::new(OpenAiTtsProvider::azure(
            &endpoint,
            &api_key,
            &deployment,
            api_version.as_deref(),
        )))
    } else {
        let api_key = std::env::var("OPENAI_API_KEY")
            .ok()
            .or_else(|| cfg.openai.api_key.clone());
        if api_key.is_none() {
            anyhow::bail!(
                "no OpenAI API key found. Set OPENAI_API_KEY (a .env file in this directory works), \
                 put it in your config file, or pass --azure for Azure OpenAI."
            );
        }
        Ok(Arc::new(OpenAiTtsProvider::standard(api_key)))
    }
}

fn default_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join("txttattler")
}

fn print_voices() {
    println!("{}", style("Available voices 🎭").bold());
    for voice in Voice::ALL {
        println!(
            "  {} {} — {}",
            style("•").cyan(),
            style(voice.as_str()).bold().cyan(),
            style(voice.blurb()).dim()
        );
    }
}

fn print_summary(options: &TtsOptions, use_azure: bool, cache_dir: &Path, no_cache: bool) {
    let backend = if use_azure { "Azure OpenAI" } else { "OpenAI" };
    println!(
        "{} {} · voice {} · model {} · speed {}x",
        style("⚙").dim(),
        style(backend).magenta(),
        style(options.voice).cyan(),
        style(options.model).cyan(),
        style(options.speed).cyan(),
    );
    if no_cache {
        println!("   {}", style("cache disabled (--no-cache)").dim());
    } else {
        println!("   {} {}", style("cache:").dim(), style(cache_dir.display()).dim());
    }
}

/// A colourful, emoji-laden [`ProgressReporter`] for interactive terminal use.
struct ConsoleReporter {
    verbose: bool,
    bar: Mutex<Option<ProgressBar>>,
}

impl ConsoleReporter {
    fn new(verbose: bool) -> Self {
        ConsoleReporter {
            verbose,
            bar: Mutex::new(None),
        }
    }
}

impl ProgressReporter for ConsoleReporter {
    fn reading(&self, path: &Path) {
        println!(
            "{} reading {}",
            style("📖").dim(),
            style(path.display()).cyan()
        );
    }

    fn text_ready(&self, chars: usize, chunks: usize) {
        println!(
            "{} {} characters → {} chunk{}",
            style("🧹").dim(),
            style(chars).bold(),
            chunks,
            plural(chunks)
        );
    }

    fn cache_hit(&self, path: &Path) {
        println!(
            "{} cache hit — replaying without bothering OpenAI",
            style("⚡").yellow()
        );
        if self.verbose {
            println!("   {}", style(path.display()).dim());
        }
    }

    fn synthesis_started(&self, chunks: usize) {
        println!(
            "{} synthesizing {} chunk{} via TTS…",
            style("🎙").dim(),
            chunks,
            plural(chunks)
        );
        let bar = ProgressBar::new(chunks as u64);
        bar.set_style(
            ProgressStyle::with_template(
                "   {spinner:.green} [{bar:30.cyan/blue}] {pos}/{len} chunks",
            )
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("=>-"),
        );
        *self.bar.lock().unwrap() = Some(bar);
    }

    fn chunk_done(&self, _done: usize, _total: usize) {
        if let Some(bar) = self.bar.lock().unwrap().as_ref() {
            bar.inc(1);
        }
    }

    fn synthesis_finished(&self) {
        if let Some(bar) = self.bar.lock().unwrap().take() {
            bar.finish_and_clear();
        }
        println!("   {}", style("audio generated").green());
    }

    fn cached(&self, path: &Path) {
        if self.verbose {
            println!("{} cached → {}", style("💾").dim(), style(path.display()).dim());
        }
    }

    fn output_saved(&self, path: &Path) {
        println!(
            "{} saved MP3 → {}",
            style("💾").dim(),
            style(path.display()).green()
        );
    }

    fn playing(&self) {
        println!("{} now playing… (Ctrl-C to stop)", style("🔊").dim());
    }

    fn finished(&self, outcome: &TtsOutcome) {
        println!(
            "{} {}",
            style("✅").green(),
            style("all done — thanks for letting me tattle!").green().bold()
        );
        if self.verbose {
            println!(
                "   {} bytes of audio · cache {}",
                outcome.audio_len,
                if outcome.cache_hit { "hit" } else { "miss" }
            );
        }
    }
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}
