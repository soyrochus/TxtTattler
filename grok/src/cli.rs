//! CLI definition for TxtTattler using clap v4 derive.
//! Beautiful help text, colors, and just the right amount of personality.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// ████████╗██╗  ██╗████████╗████████╗ █████╗ ████████╗████████╗██╗     ███████╗██████╗ 
/// ╚══██╔══╝╚██╗██╔╝╚══██╔══╝╚══██╔══╝██╔══██╗╚══██╔══╝╚══██╔══╝██║     ██╔════╝██╔══██╗
///    ██║    ╚███╔╝    ██║      ██║   ███████║   ██║      ██║   ██║     █████╗  ██████╔╝
///    ██║    ██╔██╗    ██║      ██║   ██╔══██║   ██║      ██║   ██║     ██╔══╝  ██╔══██╗
///    ██║   ██╔╝ ██╗   ██║      ██║   ██║  ██║   ██║      ██║   ███████╗███████╗██║  ██║
///    ╚═╝   ╚═╝  ╚═╝   ╚═╝      ╚═╝   ╚═╝  ╚═╝   ╚═╝      ╚═╝   ╚══════╝╚══════╝╚═╝  ╚═╝
/// 
/// The little command-line tattletale that reads your files out loud.
/// Powered by OpenAI TTS. Built with pure Rust and questionable amounts of caffeine.
///
/// "Your documents have never sounded so dramatic."
#[derive(Parser, Debug)]
#[command(name = "txttattler")]
#[command(bin_name = "txttattler")]
#[command(version, about, long_about = None)]
#[command(after_help = "EXAMPLES:\n  txttattler meeting-notes.txt\n  txttattler --voice fable --speed 0.9 novel-chapter-7.txt\n  txttattler report.pdf --output meeting.mp3 --no-play\n  txttattler long-doc.txt --refresh          # force new TTS + update cache\n  txttattler long-doc.txt --no-cache         # one-off, don't touch cache\n  txttattler --list-voices\n\nENVIRONMENT:\n  OPENAI_API_KEY          Your OpenAI key (required unless using --azure)\n  AZURE_OPENAI_API_KEY    Azure key when using --azure\n  AZURE_OPENAI_ENDPOINT   Your Azure endpoint (https://....openai.azure.com/)\n\nThe tattler is always listening. Use responsibly.")]
pub struct Cli {
    /// The file to read aloud (.txt supported today, .docx/.pdf coming soon)
    #[arg(value_name = "FILE", required_unless_present = "list_voices")]
    pub file: Option<PathBuf>,

    /// Voice personality to use for the narration
    #[arg(short, long, value_enum, default_value_t = VoiceArg::Alloy)]
    pub voice: VoiceArg,

    /// Which OpenAI TTS model to use
    #[arg(short, long, value_enum, default_value_t = ModelArg::Tts1)]
    pub model: ModelArg,

    /// Speed multiplier (0.25 = slow dramatic reading, 4.0 = chipmunk on caffeine)
    #[arg(short, long, value_name = "FLOAT", default_value_t = 1.0)]
    pub speed: f32,

    /// Save the generated audio to this path instead of (or in addition to) playing it
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Generate the file but do not play it through speakers
    #[arg(long)]
    pub no_play: bool,

    /// Disable the MP3 cache entirely for this run (always call OpenAI)
    #[arg(long)]
    pub no_cache: bool,

    /// Force regeneration even if a valid cached MP3 exists (updates the cache)
    #[arg(long)]
    pub refresh: bool,

    /// Override the default platform cache directory for MP3 files
    #[arg(long, value_name = "PATH")]
    pub cache_dir: Option<PathBuf>,

    /// Force Azure OpenAI endpoint (uses AZURE_* environment variables or config)
    #[arg(long)]
    pub azure: bool,

    /// Path to a custom config file (TOML). Defaults to ~/.config/txttattler/config.toml
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// List all available voices with fun descriptions and exit
    #[arg(long)]
    pub list_voices: bool,

    /// Enable verbose logging (great for debugging why the tattler is being dramatic).
    /// Note: -V / --version is reserved by clap for showing the program version.
    #[arg(long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Subcommands for advanced tattling
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Show current effective configuration (merged env + file + defaults)
    Config {
        /// Print as TOML instead of pretty table
        #[arg(long)]
        toml: bool,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum VoiceArg {
    Alloy,
    Echo,
    Fable,
    Onyx,
    Nova,
    Shimmer,
}

impl From<VoiceArg> for crate::domain::entities::Voice {
    fn from(v: VoiceArg) -> Self {
        match v {
            VoiceArg::Alloy => crate::domain::entities::Voice::Alloy,
            VoiceArg::Echo => crate::domain::entities::Voice::Echo,
            VoiceArg::Fable => crate::domain::entities::Voice::Fable,
            VoiceArg::Onyx => crate::domain::entities::Voice::Onyx,
            VoiceArg::Nova => crate::domain::entities::Voice::Nova,
            VoiceArg::Shimmer => crate::domain::entities::Voice::Shimmer,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum ModelArg {
    #[value(name = "tts-1")]
    Tts1,
    #[value(name = "tts-1-hd")]
    Tts1Hd,
}

impl From<ModelArg> for crate::domain::entities::TtsModel {
    fn from(m: ModelArg) -> Self {
        match m {
            ModelArg::Tts1 => crate::domain::entities::TtsModel::Tts1,
            ModelArg::Tts1Hd => crate::domain::entities::TtsModel::Tts1Hd,
        }
    }
}

/// Print the fun list of voices
pub fn print_voice_list() {
    use console::style;

    println!();
    println!("{}", style("🎙️  TXT TATTLER — AVAILABLE VOICES").bold().cyan());
    println!("{}", style("─────────────────────────────────────────────").dim());
    println!();

    let voices = [
        ("alloy", "Neutral, clear, and slightly robotic — the professional gossip"),
        ("echo", "Deep, resonant, great for dramatic readings and bedtime stories"),
        ("fable", "British storyteller energy. Perfect for novels and fairy tales"),
        ("onyx", "Deep, authoritative, slightly menacing — ideal for legal documents"),
        ("nova", "Warm, friendly, female-presenting — your favorite podcast host"),
        ("shimmer", "Soft, ethereal, dreamy — great for poetry and love letters"),
    ];

    for (name, desc) in voices {
        println!(
            "  {} {}\n    {}",
            style("•").green().bold(),
            style(name).yellow().bold(),
            style(desc).dim()
        );
    }

    println!();
    println!("{}", style("Default voice: alloy").italic().dim());
    println!("{}", style("Tip: fable at speed 0.85 is pure magic for long documents.").italic().dim());
    println!();
}
