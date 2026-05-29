use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Voice {
    Alloy,
    Echo,
    Fable,
    Onyx,
    Nova,
    Shimmer,
}

impl Voice {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Alloy => "alloy",
            Self::Echo => "echo",
            Self::Fable => "fable",
            Self::Onyx => "onyx",
            Self::Nova => "nova",
            Self::Shimmer => "shimmer",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpeechModel {
    Tts1,
    Tts1Hd,
}

impl SpeechModel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tts1 => "tts-1",
            Self::Tts1Hd => "tts-1-hd",
        }
    }
}

#[derive(Clone, Debug)]
pub struct TtsRequest {
    pub file: PathBuf,
    pub voice: Voice,
    pub model: SpeechModel,
    pub speed: f32,
    pub output: Option<PathBuf>,
    pub play: bool,
    pub use_cache: bool,
    pub refresh: bool,
    pub cache_dir: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct TtsOptions {
    pub voice: Voice,
    pub model: SpeechModel,
    pub speed: f32,
}

#[derive(Clone, Debug, Default)]
pub struct TtsResult {
    pub bytes: usize,
    pub chunks: usize,
    pub saved_to: Option<PathBuf>,
    pub cache_path: Option<PathBuf>,
    pub cache_hit: bool,
}
