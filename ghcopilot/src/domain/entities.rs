use std::{fmt, path::PathBuf, str::FromStr};

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VoiceName {
    Alloy,
    Echo,
    Fable,
    Onyx,
    Nova,
    Shimmer,
}

impl VoiceName {
    pub const ALL: [Self; 6] = [
        Self::Alloy,
        Self::Echo,
        Self::Fable,
        Self::Onyx,
        Self::Nova,
        Self::Shimmer,
    ];

    pub const fn as_str(self) -> &'static str {
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

impl Default for VoiceName {
    fn default() -> Self {
        Self::Alloy
    }
}

impl fmt::Display for VoiceName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for VoiceName {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "alloy" => Ok(Self::Alloy),
            "echo" => Ok(Self::Echo),
            "fable" => Ok(Self::Fable),
            "onyx" => Ok(Self::Onyx),
            "nova" => Ok(Self::Nova),
            "shimmer" => Ok(Self::Shimmer),
            other => Err(anyhow!("Unsupported voice '{other}'.")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpeechModelName {
    Tts1,
    Tts1Hd,
}

impl SpeechModelName {
    pub const ALL: [Self; 2] = [Self::Tts1, Self::Tts1Hd];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tts1 => "tts-1",
            Self::Tts1Hd => "tts-1-hd",
        }
    }
}

impl Default for SpeechModelName {
    fn default() -> Self {
        Self::Tts1
    }
}

impl fmt::Display for SpeechModelName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for SpeechModelName {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "tts-1" => Ok(Self::Tts1),
            "tts-1-hd" => Ok(Self::Tts1Hd),
            other => Err(anyhow!("Unsupported model '{other}'.")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtsRequest {
    pub text: String,
    pub voice: VoiceName,
    pub model: SpeechModelName,
    pub speed: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynthesisOutcome {
    pub cache_path: Option<PathBuf>,
    pub from_cache: bool,
    pub output_path: Option<PathBuf>,
    pub chunks: usize,
    pub character_count: usize,
}

#[cfg(test)]
mod tests {
    use super::{SpeechModelName, VoiceName};
    use std::str::FromStr;

    #[test]
    fn voice_parses_case_insensitively() {
        assert_eq!(VoiceName::from_str("NoVa").unwrap(), VoiceName::Nova);
    }

    #[test]
    fn model_parses_case_insensitively() {
        assert_eq!(
            SpeechModelName::from_str("TTS-1-HD").unwrap(),
            SpeechModelName::Tts1Hd
        );
    }
}
