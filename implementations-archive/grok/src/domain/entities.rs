//! Domain entities for TxtTattler.
//! These are the gossipy little data structures that carry your words to the world.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

/// Supported OpenAI TTS voices (the six gossiping personalities)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
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

    #[allow(dead_code)]
    pub fn all() -> &'static [Voice] {
        &[
            Voice::Alloy,
            Voice::Echo,
            Voice::Fable,
            Voice::Onyx,
            Voice::Nova,
            Voice::Shimmer,
        ]
    }
}

impl fmt::Display for Voice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Voice {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "alloy" => Ok(Voice::Alloy),
            "echo" => Ok(Voice::Echo),
            "fable" => Ok(Voice::Fable),
            "onyx" => Ok(Voice::Onyx),
            "nova" => Ok(Voice::Nova),
            "shimmer" => Ok(Voice::Shimmer),
            _ => Err(anyhow::anyhow!("Unknown voice '{}'. Valid: alloy, echo, fable, onyx, nova, shimmer", s)),
        }
    }
}

/// TTS model choice — tts-1 is fast & cheap, tts-1-hd is the fancy gossiper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
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

impl fmt::Display for TtsModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for TtsModel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tts-1" | "tts1" => Ok(TtsModel::Tts1),
            "tts-1-hd" | "tts1hd" | "hd" => Ok(TtsModel::Tts1Hd),
            _ => Err(anyhow::anyhow!("Unknown model '{}'. Use tts-1 or tts-1-hd", s)),
        }
    }
}

/// Configuration options for a single TTS synthesis request.
/// The tattler uses these to decide how dramatic to be.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsOptions {
    pub voice: Voice,
    pub model: TtsModel,
    /// 0.25 to 4.0 — how fast the gossip spreads
    pub speed: f32,
}

impl Default for TtsOptions {
    fn default() -> Self {
        Self {
            voice: Voice::default(),
            model: TtsModel::default(),
            speed: 1.0,
        }
    }
}

impl TtsOptions {
    pub fn new(voice: Voice, model: TtsModel, speed: f32) -> anyhow::Result<Self> {
        if !(0.25..=4.0).contains(&speed) {
            anyhow::bail!("Speed must be between 0.25 and 4.0 (you asked for {})", speed);
        }
        Ok(Self { voice, model, speed })
    }
}

/// Represents a chunk of text ready for TTS.
/// OpenAI has a soft ~4096 character limit per request. We respect it like polite gossips.
#[derive(Debug, Clone)]
pub struct TextChunk {
    pub index: usize,
    pub text: String,
    pub char_count: usize,
}

impl TextChunk {
    pub fn new(index: usize, text: String) -> Self {
        let char_count = text.chars().count();
        Self {
            index,
            text,
            char_count,
        }
    }
}

/// Result of a full TTS run. Contains metadata the tattler loves to brag about.
#[derive(Debug, Clone)]
pub struct TtsResult {
    pub total_chunks: usize,
    pub total_chars: usize,
    pub voice_used: Voice,
    pub model_used: TtsModel,
    /// Path to the final combined audio if --output was used
    pub output_path: Option<std::path::PathBuf>,
    #[allow(dead_code)]
    pub duration_hint_secs: Option<f32>,
}

/// A processed file ready for tattling.
/// In future versions this will carry provenance (which page of the PDF etc.)
#[derive(Debug, Clone)]
pub struct ProcessedDocument {
    #[allow(dead_code)]
    pub source_path: std::path::PathBuf,
    pub original_filename: String,
    pub plain_text: String,
    pub char_count: usize,
    pub word_count: usize,
}

impl ProcessedDocument {
    pub fn new(source: &Path, text: String) -> Self {
        let char_count = text.chars().count();
        let word_count = text.split_whitespace().count();
        let original_filename = source
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "unknown".to_string());

        Self {
            source_path: source.to_path_buf(),
            original_filename,
            plain_text: text,
            char_count,
            word_count,
        }
    }

    /// Split into chunks respecting OpenAI's limits while trying to keep sentences whole.
    /// The tattler is smart — it doesn't cut mid-gossip.
    pub fn chunk(&self, max_chars: usize) -> Vec<TextChunk> {
        if self.plain_text.is_empty() {
            return vec![];
        }

        if self.char_count <= max_chars {
            return vec![TextChunk::new(0, self.plain_text.clone())];
        }

        let mut chunks = Vec::new();
        let mut current = String::new();
        let mut idx = 0;

        // Split on sentence boundaries where possible, fall back to word, then hard cut
        for sentence in self.plain_text.split(|c| c == '.' || c == '!' || c == '?') {
            let mut candidate = sentence.to_string();
            // Re-add punctuation crudely (we lost it in split)
            if !candidate.trim().is_empty() {
                // crude restore
                if self.plain_text.contains(&format!("{}.", candidate.trim())) {
                    candidate.push('.');
                }
            }

            if current.len() + candidate.len() + 1 > max_chars && !current.is_empty() {
                chunks.push(TextChunk::new(idx, std::mem::take(&mut current).trim().to_string()));
                idx += 1;
            }

            if candidate.len() > max_chars {
                // Hard split the monster sentence
                for part in candidate.as_bytes().chunks(max_chars) {
                    if let Ok(s) = std::str::from_utf8(part) {
                        if !s.trim().is_empty() {
                            chunks.push(TextChunk::new(idx, s.trim().to_string()));
                            idx += 1;
                        }
                    }
                }
                current.clear();
            } else if !candidate.trim().is_empty() {
                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(candidate.trim());
            }
        }

        if !current.trim().is_empty() {
            chunks.push(TextChunk::new(idx, current.trim().to_string()));
        }

        // Fallback: if still empty or single huge, force split
        if chunks.is_empty() {
            for (i, part) in self.plain_text.as_bytes().chunks(max_chars).enumerate() {
                if let Ok(s) = std::str::from_utf8(part) {
                    chunks.push(TextChunk::new(i, s.to_string()));
                }
            }
        }

        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_parsing_roundtrips() {
        for v in Voice::all() {
            let s = v.as_str();
            let parsed: Voice = s.parse().unwrap();
            assert_eq!(*v, parsed);
        }
    }

    #[test]
    fn chunking_respects_limit() {
        let long_text = (0..80)
            .map(|i| format!("This is sentence number {} about the tattler and its many secrets. ", i))
            .collect::<String>();
        let doc = ProcessedDocument::new(std::path::Path::new("test.txt"), long_text);
        let chunks = doc.chunk(300);
        assert!(chunks.len() > 1, "expected multiple chunks");
        for c in &chunks {
            assert!(c.char_count <= 320, "chunk {} too large", c.char_count);
        }
    }

    #[test]
    fn tts_options_validates_speed() {
        assert!(TtsOptions::new(Voice::Alloy, TtsModel::Tts1, 0.1).is_err());
        assert!(TtsOptions::new(Voice::Alloy, TtsModel::Tts1, 5.0).is_err());
        assert!(TtsOptions::new(Voice::Alloy, TtsModel::Tts1, 1.5).is_ok());
    }
}
