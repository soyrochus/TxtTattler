use std::path::PathBuf;

use clap::{Parser, ValueEnum};

use crate::domain::entities::{SpeechModel, TtsRequest, Voice};

pub const VOICES: &[&str] = &["alloy", "echo", "fable", "onyx", "nova", "shimmer"];

#[derive(Debug, Parser)]
#[command(
    name = "txttattler",
    version,
    disable_version_flag = true,
    about = "TxtTattler tattles your text files out loud with OpenAI TTS.",
    long_about = "TxtTattler is a gossipy little CLI that reads plain text files aloud using OpenAI or Azure OpenAI TTS.",
    after_help = "Examples:\n  txttattler notes.txt\n  txttattler notes.txt --voice nova --speed 1.2\n  txttattler notes.txt --output notes.mp3 --no-play\n  txttattler notes.txt --refresh"
)]
pub struct Cli {
    /// File to read aloud. Version 1 supports UTF-8 .txt files.
    pub file: Option<PathBuf>,

    /// Voice to use.
    #[arg(short, long, value_enum, default_value_t = VoiceArg::Alloy)]
    pub voice: VoiceArg,

    /// TTS model to use.
    #[arg(short, long, value_enum, default_value_t = ModelArg::Tts1)]
    pub model: ModelArg,

    /// Playback speed from 0.25 to 4.0.
    #[arg(short, long, default_value_t = 1.0, value_parser = parse_speed)]
    pub speed: f32,

    /// Save generated MP3 to this path.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Generate the MP3 but do not play it.
    #[arg(long)]
    pub no_play: bool,

    /// Do not read from or write to the automatic MP3 cache.
    #[arg(long)]
    pub no_cache: bool,

    /// Ignore any cached MP3 and regenerate speech.
    #[arg(long)]
    pub refresh: bool,

    /// Override the default MP3 cache directory.
    #[arg(long)]
    pub cache_dir: Option<PathBuf>,

    /// Force Azure OpenAI settings.
    #[arg(long)]
    pub azure: bool,

    /// Path to optional TOML config file.
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Print available voices and exit.
    #[arg(long)]
    pub list_voices: bool,

    /// Enable verbose logging.
    #[arg(short = 'V', long)]
    pub verbose: bool,
}

fn parse_speed(value: &str) -> Result<f32, String> {
    let speed = value
        .parse::<f32>()
        .map_err(|_| "speed must be a number between 0.25 and 4.0".to_owned())?;

    if (0.25..=4.0).contains(&speed) {
        Ok(speed)
    } else {
        Err("speed must be between 0.25 and 4.0".to_owned())
    }
}

impl Cli {
    pub fn into_request(self) -> TtsRequest {
        TtsRequest {
            file: self.file.unwrap_or_else(|| PathBuf::from("")),
            voice: self.voice.into(),
            model: self.model.into(),
            speed: self.speed,
            output: self.output,
            play: !self.no_play,
            use_cache: !self.no_cache,
            refresh: self.refresh,
            cache_dir: self.cache_dir,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum VoiceArg {
    Alloy,
    Echo,
    Fable,
    Onyx,
    Nova,
    Shimmer,
}

impl std::fmt::Display for VoiceArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Alloy => "alloy",
            Self::Echo => "echo",
            Self::Fable => "fable",
            Self::Onyx => "onyx",
            Self::Nova => "nova",
            Self::Shimmer => "shimmer",
        };
        f.write_str(value)
    }
}

impl From<VoiceArg> for Voice {
    fn from(value: VoiceArg) -> Self {
        match value {
            VoiceArg::Alloy => Voice::Alloy,
            VoiceArg::Echo => Voice::Echo,
            VoiceArg::Fable => Voice::Fable,
            VoiceArg::Onyx => Voice::Onyx,
            VoiceArg::Nova => Voice::Nova,
            VoiceArg::Shimmer => Voice::Shimmer,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ModelArg {
    #[value(name = "tts-1")]
    Tts1,
    #[value(name = "tts-1-hd")]
    Tts1Hd,
}

impl std::fmt::Display for ModelArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tts1 => f.write_str("tts-1"),
            Self::Tts1Hd => f.write_str("tts-1-hd"),
        }
    }
}

impl From<ModelArg> for SpeechModel {
    fn from(value: ModelArg) -> Self {
        match value {
            ModelArg::Tts1 => SpeechModel::Tts1,
            ModelArg::Tts1Hd => SpeechModel::Tts1Hd,
        }
    }
}
