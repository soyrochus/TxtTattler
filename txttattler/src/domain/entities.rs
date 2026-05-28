use std::{fmt, path::PathBuf, str::FromStr};

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VoiceName {
    Alloy,
    Ash,
    Ballad,
    Coral,
    Echo,
    Fable,
    Onyx,
    Nova,
    Sage,
    Shimmer,
    Verse,
    Marin,
    Cedar,
}

impl VoiceName {
    pub const ALL: [Self; 13] = [
        Self::Alloy,
        Self::Ash,
        Self::Ballad,
        Self::Coral,
        Self::Echo,
        Self::Fable,
        Self::Onyx,
        Self::Nova,
        Self::Sage,
        Self::Shimmer,
        Self::Verse,
        Self::Marin,
        Self::Cedar,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Alloy => "alloy",
            Self::Ash => "ash",
            Self::Ballad => "ballad",
            Self::Coral => "coral",
            Self::Echo => "echo",
            Self::Fable => "fable",
            Self::Onyx => "onyx",
            Self::Nova => "nova",
            Self::Sage => "sage",
            Self::Shimmer => "shimmer",
            Self::Verse => "verse",
            Self::Marin => "marin",
            Self::Cedar => "cedar",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Alloy => "balanced and versatile",
            Self::Ash => "warm, grounded, and conversational",
            Self::Ballad => "melodic and expressive",
            Self::Coral => "clear, bright, and friendly",
            Self::Echo => "clear, crisp narration",
            Self::Fable => "warm storytelling tone",
            Self::Onyx => "deep and steady",
            Self::Nova => "bright and expressive",
            Self::Sage => "calm, thoughtful, and measured",
            Self::Shimmer => "soft and polished",
            Self::Verse => "smooth and dynamic",
            Self::Marin => "natural, relaxed, and approachable",
            Self::Cedar => "steady, resonant, and composed",
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
            "ash" => Ok(Self::Ash),
            "ballad" => Ok(Self::Ballad),
            "coral" => Ok(Self::Coral),
            "echo" => Ok(Self::Echo),
            "fable" => Ok(Self::Fable),
            "onyx" => Ok(Self::Onyx),
            "nova" => Ok(Self::Nova),
            "sage" => Ok(Self::Sage),
            "shimmer" => Ok(Self::Shimmer),
            "verse" => Ok(Self::Verse),
            "marin" => Ok(Self::Marin),
            "cedar" => Ok(Self::Cedar),
            other => Err(anyhow!("Unsupported voice '{other}'.")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpeechModelName {
    Tts1,
    Tts1Hd,
    Gpt4oMiniTts,
}

impl SpeechModelName {
    pub const ALL: [Self; 3] = [Self::Tts1, Self::Tts1Hd, Self::Gpt4oMiniTts];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tts1 => "tts-1",
            Self::Tts1Hd => "tts-1-hd",
            Self::Gpt4oMiniTts => "gpt-4o-mini-tts",
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
            "gpt-4o-mini-tts" => Ok(Self::Gpt4oMiniTts),
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
    pub instructions: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SynthesisOutcome {
    pub input_path: PathBuf,
    pub cache_path: Option<PathBuf>,
    pub from_cache: bool,
    pub output_path: Option<PathBuf>,
    pub chunks: usize,
    pub character_count: usize,
    pub voice: VoiceName,
    pub model: SpeechModelName,
    pub speed: f32,
    pub instructions: Option<String>,
    pub playback: bool,
}

#[cfg(test)]
mod tests {
    use super::{SpeechModelName, VoiceName};
    use std::str::FromStr;

    #[test]
    fn voice_parses_case_insensitively() {
        assert_eq!(VoiceName::from_str("NoVa").unwrap(), VoiceName::Nova);
        assert_eq!(VoiceName::from_str("CORAL").unwrap(), VoiceName::Coral);
    }

    #[test]
    fn model_parses_case_insensitively() {
        assert_eq!(
            SpeechModelName::from_str("TTS-1-HD").unwrap(),
            SpeechModelName::Tts1Hd
        );
        assert_eq!(
            SpeechModelName::from_str("GPT-4O-MINI-TTS").unwrap(),
            SpeechModelName::Gpt4oMiniTts
        );
    }

    #[test]
    fn gpt4o_mini_tts_displays_as_api_name() {
        assert_eq!(SpeechModelName::Gpt4oMiniTts.to_string(), "gpt-4o-mini-tts");
    }

    #[test]
    fn all_model_names_contains_three_models() {
        assert_eq!(SpeechModelName::ALL.len(), 3);
    }

    #[test]
    fn extended_voices_round_trip_and_have_descriptions() {
        for voice in [
            VoiceName::Ash,
            VoiceName::Ballad,
            VoiceName::Coral,
            VoiceName::Sage,
            VoiceName::Verse,
            VoiceName::Marin,
            VoiceName::Cedar,
        ] {
            assert_eq!(VoiceName::from_str(voice.as_str()).unwrap(), voice);
            assert!(!voice.description().is_empty());
        }
    }

    #[test]
    fn all_voice_names_contains_thirteen_voices() {
        assert_eq!(VoiceName::ALL.len(), 13);
    }
}
