//! Command-line surface, powered by clap's derive macros.

use std::path::PathBuf;

use clap::Parser;

/// The cheeky banner shown above `--help`.
pub const BANNER: &str = r#"
  _______        _ _____      _   _
 |__   __|      | |_   _|    | | | |
    | |_  _| |_| |_| | __ _| |_| |_ ___ _ __
    | \ \/ / __| __| |/ _` | __| __/ _ \ '__|
    | |>  <| |_| |_| | (_| | |_| ||  __/ |
    |_/_/\_\\__|\__|_|\__,_|\__|\__\___|_|

  TxtTattler 🗣️  — the little gossip that reads your files out loud.
"#;

/// TxtTattler reads your text files aloud using OpenAI's text-to-speech. 🎧
#[derive(Parser, Debug)]
#[command(
    name = "txttattler",
    version = "1.0.0",
    about = "TxtTattler 🗣️  — reads your text files aloud with OpenAI TTS",
    before_help = BANNER,
    disable_version_flag = true
)]
pub struct Cli {
    /// The file to read aloud (.txt today; .docx and .pdf are coming soon).
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Voice: alloy | echo | fable | onyx | nova | shimmer.
    #[arg(short, long, default_value = "alloy", value_name = "VOICE")]
    pub voice: String,

    /// TTS model: tts-1 (fast) | tts-1-hd (higher quality).
    #[arg(short, long, default_value = "tts-1", value_name = "MODEL")]
    pub model: String,

    /// Playback speed, 0.25–4.0.
    #[arg(short, long, default_value_t = 1.0_f32, value_name = "SPEED")]
    pub speed: f32,

    /// Save the generated MP3 to this path (in addition to caching/playing).
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Generate (and cache/save) the audio but don't play it.
    #[arg(long)]
    pub no_play: bool,

    /// Don't read from or write to the automatic MP3 cache.
    #[arg(long)]
    pub no_cache: bool,

    /// Ignore any cached MP3 and regenerate the speech.
    #[arg(long)]
    pub refresh: bool,

    /// Override the default MP3 cache directory.
    #[arg(long, value_name = "PATH")]
    pub cache_dir: Option<PathBuf>,

    /// Force the Azure OpenAI endpoint.
    #[arg(long)]
    pub azure: bool,

    /// Path to an optional TOML config file.
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Print the available voices and exit.
    #[arg(long)]
    pub list_voices: bool,

    /// Chatty diagnostic output (timings, paths, counts).
    #[arg(short = 'V', long)]
    pub verbose: bool,

    /// Print version and exit.
    #[arg(long = "version", action = clap::ArgAction::Version)]
    pub _version: (),
}
