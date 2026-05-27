//! Ports — the traits the use-case depends on. Adapters in `infrastructure`
//! plug into these holes. Swap an adapter, keep the use-case untouched.

use std::path::Path;

use async_trait::async_trait;

use crate::domain::entities::TtsOptions;

/// Turns a file on disk into a `String` of text to be spoken.
///
/// One implementor per file format (`.txt`, and later `.docx`, `.pdf`, …).
pub trait FileReader: Send + Sync {
    fn read(&self, path: &Path) -> anyhow::Result<String>;
}

/// Cleans/normalizes raw text before it is handed to the TTS engine.
///
/// Injectable middleware: chain or replace it to strip markdown, expand
/// abbreviations, etc., without touching the rest of the pipeline.
pub trait TextProcessor: Send + Sync {
    fn process(&self, text: &str) -> String;
}

/// Synthesizes a chunk of text into MP3 bytes.
#[async_trait]
pub trait TtsProvider: Send + Sync {
    async fn synthesize(&self, text: &str, options: &TtsOptions) -> anyhow::Result<Vec<u8>>;
}

/// Plays MP3 bytes through the system's speakers.
pub trait AudioPlayer: Send + Sync {
    fn play(&self, audio: &[u8]) -> anyhow::Result<()>;
}
