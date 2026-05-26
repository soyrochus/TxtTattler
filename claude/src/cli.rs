use std::path::PathBuf;

use clap::Parser;

/// TxtTattler – a fun, blazing-fast CLI text-to-speech app powered by OpenAI TTS.
#[derive(Parser, Debug)]
#[command(
    name = "txttattler",
    version = "1.0.0",
    about = "TxtTattler – reads your text files aloud using OpenAI TTS",
    disable_version_flag = true
)]
pub struct Cli {
    /// Input text file to read aloud (txt, docx, pdf).
    pub file: Option<PathBuf>,

    /// Voice to use for synthesis.
    #[arg(short, long, default_value = "alloy")]
    pub voice: String,

    /// TTS model to use.
    #[arg(short, long, default_value = "tts-1")]
    pub model: String,

    /// Speech speed (0.25–4.0).
    #[arg(short, long, default_value_t = 1.0f32)]
    pub speed: f32,

    /// Save the generated audio to this file path.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Do not play audio, only synthesize (and optionally save).
    #[arg(long)]
    pub no_play: bool,

    /// Do not use or write the local cache.
    #[arg(long)]
    pub no_cache: bool,

    /// Force re-synthesis even if a cached audio file exists.
    #[arg(long)]
    pub refresh: bool,

    /// Override the cache directory.
    #[arg(long)]
    pub cache_dir: Option<PathBuf>,

    /// Use Azure OpenAI instead of standard OpenAI.
    #[arg(long)]
    pub azure: bool,

    /// Path to a TOML config file.
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// List all available voices and exit.
    #[arg(long)]
    pub list_voices: bool,

    /// Print version and exit.
    #[arg(short = 'V', long = "version", action = clap::ArgAction::Version)]
    pub _version: (),

    /// Enable verbose/debug output.
    #[arg(long)]
    pub verbose: bool,
}
