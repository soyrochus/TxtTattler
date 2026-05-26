use std::fmt;

/// Available TTS voices.
#[derive(Debug, Clone, PartialEq)]
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
            Voice::Alloy => "alloy",
            Voice::Echo => "echo",
            Voice::Fable => "fable",
            Voice::Onyx => "onyx",
            Voice::Nova => "nova",
            Voice::Shimmer => "shimmer",
        }
    }

    pub fn from_str(s: &str) -> anyhow::Result<Self> {
        match s.to_lowercase().as_str() {
            "alloy" => Ok(Voice::Alloy),
            "echo" => Ok(Voice::Echo),
            "fable" => Ok(Voice::Fable),
            "onyx" => Ok(Voice::Onyx),
            "nova" => Ok(Voice::Nova),
            "shimmer" => Ok(Voice::Shimmer),
            other => Err(anyhow::anyhow!(
                "Unknown voice '{}'. Valid voices: alloy, echo, fable, onyx, nova, shimmer",
                other
            )),
        }
    }

    pub fn all() -> &'static [&'static str] {
        &["alloy", "echo", "fable", "onyx", "nova", "shimmer"]
    }
}

impl fmt::Display for Voice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Available TTS models.
#[derive(Debug, Clone, PartialEq)]
pub enum TtsModel {
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

    pub fn from_str(s: &str) -> anyhow::Result<Self> {
        match s.to_lowercase().as_str() {
            "tts-1" | "tts1" => Ok(TtsModel::Tts1),
            "tts-1-hd" | "tts1hd" => Ok(TtsModel::Tts1Hd),
            other => Err(anyhow::anyhow!(
                "Unknown TTS model '{}'. Valid models: tts-1, tts-1-hd",
                other
            )),
        }
    }
}

impl fmt::Display for TtsModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for TtsModel {
    fn default() -> Self {
        TtsModel::Tts1
    }
}

/// Options controlling how TTS is performed.
#[derive(Debug, Clone)]
pub struct TtsOptions {
    pub voice: Voice,
    pub model: TtsModel,
    pub speed: f32,
}

impl Default for TtsOptions {
    fn default() -> Self {
        TtsOptions {
            voice: Voice::Alloy,
            model: TtsModel::Tts1,
            speed: 1.0,
        }
    }
}


/// Newtype wrapper for a cache key string.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey(pub String);

impl fmt::Display for CacheKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
