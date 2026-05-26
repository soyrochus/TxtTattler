use std::path::Path;

use anyhow::Result;
use async_trait::async_trait;

use crate::domain::entities::TtsOptions;

pub trait FileReader: Send + Sync {
    fn read(&self, path: &Path) -> Result<String>;
}

pub trait TextProcessor: Send + Sync {
    fn process(&self, text: String) -> Result<String>;
}

#[async_trait]
pub trait TtsProvider: Send + Sync {
    async fn synthesize(&self, text: &str, options: &TtsOptions) -> Result<Vec<u8>>;
}

pub trait AudioPlayer: Send + Sync {
    fn play(&self, mp3_bytes: &[u8]) -> Result<()>;
}
