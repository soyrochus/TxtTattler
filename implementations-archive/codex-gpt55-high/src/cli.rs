use std::path::PathBuf;

use clap::Parser;

use crate::domain::entities::{Model, Voice};

const ABOUT: &str = r#"
 _____      _   _____     _   _   _
|_   _|_  _| |_|_   _|_ _| |_| |_| | ___ _ __
  | | \ \/ / __|| |/ _` | __| __| |/ _ \ '__|
  | |  >  <| |_ | | (_| | |_| |_| |  __/ |
  |_| /_/\_\\__||_|\__,_|\__|\__|_|\___|_|

A gossipy little CLI that reads your text files aloud. 🗣️
"#;

#[derive(Debug, Clone, Parser)]
#[command(
    name = "txttattler",
    version,
    about = ABOUT,
    disable_version_flag = true,
    color = clap::ColorChoice::Auto
)]
pub struct Cli {
    /// File to read aloud. Version 1 supports UTF-8-ish .txt files.
    #[arg(value_name = "FILE", required_unless_present = "list_voices")]
    pub file: Option<PathBuf>,

    /// TTS voice.
    #[arg(short, long, value_enum, default_value_t = Voice::Alloy)]
    pub voice: Voice,

    /// OpenAI TTS model.
    #[arg(short, long, value_enum, default_value = "tts-1")]
    pub model: Model,

    /// Playback speed, from 0.25 to 4.0.
    #[arg(short, long, default_value_t = 1.0, value_parser = parse_speed)]
    pub speed: f32,

    /// Save generated MP3 to this path.
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Generate the MP3 but skip playback.
    #[arg(long)]
    pub no_play: bool,

    /// Disable automatic MP3 cache reads and writes.
    #[arg(long)]
    pub no_cache: bool,

    /// Ignore existing cached MP3 and regenerate speech.
    #[arg(long)]
    pub refresh: bool,

    /// Override the platform cache directory.
    #[arg(long, value_name = "PATH")]
    pub cache_dir: Option<PathBuf>,

    /// Force Azure OpenAI mode.
    #[arg(long)]
    pub azure: bool,

    /// Optional TOML config file.
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Print available voices and exit.
    #[arg(long)]
    pub list_voices: bool,

    /// Verbose diagnostics.
    #[arg(short = 'V', long)]
    pub verbose: bool,
}

fn parse_speed(value: &str) -> Result<f32, String> {
    let parsed: f32 = value
        .parse()
        .map_err(|_| "speed must be a floating point number".to_string())?;
    if (0.25..=4.0).contains(&parsed) {
        Ok(parsed)
    } else {
        Err("speed must be between 0.25 and 4.0".to_string())
    }
}
