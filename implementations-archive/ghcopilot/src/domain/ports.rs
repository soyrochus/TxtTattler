use std::path::Path;

use anyhow::Result;
use async_trait::async_trait;

use crate::domain::entities::TtsRequest;

pub trait FileReader: Send + Sync {
    fn read(&self, path: &Path) -> Result<String>;
}

#[async_trait]
pub trait TtsProvider: Send + Sync {
    async fn synthesize(&self, request: &TtsRequest) -> Result<Vec<u8>>;
}

pub trait AudioPlayer: Send + Sync {
    fn play_mp3(&self, audio: Vec<u8>) -> Result<()>;
}

pub trait TextProcessor: Send + Sync {
    fn name(&self) -> &'static str;
    fn process(&self, input: String) -> Result<String>;
}

pub trait ProgressHandle: Send {
    fn inc(&self, delta: u64);
    fn set_message(&self, message: &str);
    fn finish_with_message(&self, message: &str);
}

pub trait Reporter: Send + Sync {
    fn status(&self, message: &str);
    fn detail(&self, message: &str);
    fn success(&self, message: &str);
    fn progress(&self, message: &str, len: u64) -> Box<dyn ProgressHandle>;
}
