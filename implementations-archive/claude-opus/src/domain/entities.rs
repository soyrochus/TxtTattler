//! Plain, framework-free domain types. The vocabulary the whole app speaks.

use std::fmt;
use std::str::FromStr;

/// One of OpenAI's six TTS voices. The tattler can gossip in any of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Voice {
    #[default]
    Alloy,
    Echo,
    Fable,
    Onyx,
    Nova,
    Shimmer,
}

impl Voice {
    /// The wire name OpenAI expects.
    pub fn as_str(&self) -> &'static str {
        match self {
            Voice::Alloy => "alloy",
            Voice::Echo => "echo",
            Voice::Fable => "fable",
            Voice::Onyx => "onyx",
            Voice::Nova => "nova",
            Voice::Shimmer => "shimmer",
        }
    }

    /// Every voice, in menu order — handy for `--list-voices`.
    pub const ALL: [Voice; 6] = [
        Voice::Alloy,
        Voice::Echo,
        Voice::Fable,
        Voice::Onyx,
        Voice::Nova,
        Voice::Shimmer,
    ];

    /// A one-line personality blurb, purely for fun.
    pub fn blurb(&self) -> &'static str {
        match self {
            Voice::Alloy => "balanced and even — the reliable narrator",
            Voice::Echo => "calm and measured",
            Voice::Fable => "warm and story-time cozy",
            Voice::Onyx => "deep and dramatic",
            Voice::Nova => "bright and energetic",
            Voice::Shimmer => "soft and silky",
        }
    }
}

impl FromStr for Voice {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "alloy" => Ok(Voice::Alloy),
            "echo" => Ok(Voice::Echo),
            "fable" => Ok(Voice::Fable),
            "onyx" => Ok(Voice::Onyx),
            "nova" => Ok(Voice::Nova),
            "shimmer" => Ok(Voice::Shimmer),
            other => Err(anyhow::anyhow!(
                "unknown voice '{other}'. Try one of: alloy, echo, fable, onyx, nova, shimmer"
            )),
        }
    }
}

impl fmt::Display for Voice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Which TTS model to use. `tts-1` is fast; `tts-1-hd` sounds nicer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TtsModel {
    #[default]
    Tts1,
    Tts1Hd,
}

impl TtsModel {
    pub fn as_str(&self) -> &'static str {
        match self {
            TtsModel::Tts1 => "tts-1",
            TtsModel::Tts1Hd => "tts-1-hd",
        }
    }
}

impl FromStr for TtsModel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "tts-1" | "tts1" => Ok(TtsModel::Tts1),
            "tts-1-hd" | "tts1hd" | "tts-1hd" => Ok(TtsModel::Tts1Hd),
            other => Err(anyhow::anyhow!(
                "unknown model '{other}'. Try one of: tts-1, tts-1-hd"
            )),
        }
    }
}

impl fmt::Display for TtsModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The lowest and highest speeds OpenAI accepts.
pub const SPEED_MIN: f32 = 0.25;
pub const SPEED_MAX: f32 = 4.0;

/// Everything that influences the *sound* of a synthesis request.
///
/// These four fields are exactly what the cache key is built from, so two
/// requests with identical options + identical text produce the same audio.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TtsOptions {
    pub voice: Voice,
    pub model: TtsModel,
    pub speed: f32,
}

impl TtsOptions {
    /// Reject speeds outside the supported range early, with a friendly message.
    pub fn validate(&self) -> anyhow::Result<()> {
        if !(SPEED_MIN..=SPEED_MAX).contains(&self.speed) {
            anyhow::bail!(
                "speed must be between {SPEED_MIN} and {SPEED_MAX} (got {})",
                self.speed
            );
        }
        Ok(())
    }
}

impl Default for TtsOptions {
    fn default() -> Self {
        TtsOptions {
            voice: Voice::default(),
            model: TtsModel::default(),
            speed: 1.0,
        }
    }
}

/// A content-addressed cache key (a hex SHA-256 digest of text + options + version).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey(pub String);

impl CacheKey {
    /// The cache filename this key maps to.
    pub fn file_name(&self) -> String {
        format!("{}.mp3", self.0)
    }
}

impl fmt::Display for CacheKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_round_trips_through_strings() {
        for voice in Voice::ALL {
            let parsed: Voice = voice.as_str().parse().unwrap();
            assert_eq!(voice, parsed);
        }
    }

    #[test]
    fn voice_parsing_is_case_insensitive() {
        assert_eq!("AlLoY".parse::<Voice>().unwrap(), Voice::Alloy);
        assert_eq!("  nova ".parse::<Voice>().unwrap(), Voice::Nova);
    }

    #[test]
    fn unknown_voice_is_rejected() {
        assert!("clippy".parse::<Voice>().is_err());
    }

    #[test]
    fn model_accepts_friendly_aliases() {
        assert_eq!("tts1".parse::<TtsModel>().unwrap(), TtsModel::Tts1);
        assert_eq!("TTS-1-HD".parse::<TtsModel>().unwrap(), TtsModel::Tts1Hd);
    }

    #[test]
    fn speed_validation_enforces_bounds() {
        let mut opts = TtsOptions::default();
        assert!(opts.validate().is_ok());
        opts.speed = 0.1;
        assert!(opts.validate().is_err());
        opts.speed = 5.0;
        assert!(opts.validate().is_err());
    }
}
