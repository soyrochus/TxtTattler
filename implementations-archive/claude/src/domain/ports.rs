use std::path::Path;

use async_trait::async_trait;

use crate::domain::entities::TtsOptions;

/// Reads text content from a file.
pub trait FileReader: Send + Sync {
    fn read(&self, path: &Path) -> anyhow::Result<String>;
}

/// Synthesizes text to audio bytes using a TTS service.
#[async_trait]
pub trait TtsProvider: Send + Sync {
    async fn synthesize(&self, text: &str, options: &TtsOptions) -> anyhow::Result<Vec<u8>>;
}

/// Plays raw audio bytes through the system audio output.
pub trait AudioPlayer: Send + Sync {
    fn play(&self, audio_data: &[u8]) -> anyhow::Result<()>;
}

/// Processes and normalizes text before synthesis.
pub trait TextProcessor: Send + Sync {
    fn process(&self, text: &str) -> String;
}
