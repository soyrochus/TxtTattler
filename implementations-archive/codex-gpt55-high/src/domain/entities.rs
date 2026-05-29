use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

pub const TEXT_PROCESSING_VERSION: &str = "txttattler-text-v1";
pub const MAX_TTS_CHARS: usize = 4096;
pub const AVAILABLE_VOICES: &[Voice] = &[
    Voice::Alloy,
    Voice::Echo,
    Voice::Fable,
    Voice::Onyx,
    Voice::Nova,
    Voice::Shimmer,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Voice {
    Alloy,
    Echo,
    Fable,
    Onyx,
    Nova,
    Shimmer,
}

impl Display for Voice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Voice::Alloy => "alloy",
            Voice::Echo => "echo",
            Voice::Fable => "fable",
            Voice::Onyx => "onyx",
            Voice::Nova => "nova",
            Voice::Shimmer => "shimmer",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Model {
    #[value(name = "tts-1")]
    Tts1,
    #[value(name = "tts-1-hd")]
    Tts1Hd,
}

impl Display for Model {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Model::Tts1 => "tts-1",
            Model::Tts1Hd => "tts-1-hd",
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TtsOptions {
    pub voice: Voice,
    pub model: Model,
    pub speed: f32,
}

#[derive(Debug, Clone)]
pub struct ProcessedText {
    pub original_character_count: usize,
    pub cleaned: String,
}

#[derive(Debug, Clone)]
pub struct SpeechReport {
    pub character_count: usize,
    pub chunk_count: usize,
    pub output_path: Option<PathBuf>,
    pub cache_path: Option<PathBuf>,
    pub cache_hit: bool,
}
