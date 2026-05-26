//! Ports (traits) — the contract that every tattler adapter must swear to uphold.
//!
//! Following the sacred hexagonal architecture: the domain tells the outside world
//! "this is what I need", and the infrastructure says "here, I brought it".

use super::entities::{ProcessedDocument, TextChunk, TtsOptions, TtsResult};
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

/// The FileReader port. Any gossip source (txt today, docx tomorrow, PDF the day after)
/// must implement this so the core use-case stays blissfully ignorant of formats.
pub trait FileReader: Send + Sync {
    /// Read the file at `path` and return clean UTF-8 text.
    fn read(&self, path: &Path) -> Result<String>;

    /// Which file extensions this reader claims responsibility for (e.g. [".txt"])
    fn supported_extensions(&self) -> &'static [&'static str];

    /// Human friendly name for logging ("Plain Text Tattler")
    fn name(&self) -> &'static str;
}

/// Registry that maps extensions to readers. Adding .docx later = one line change.
/// The core never needs to know.
#[derive(Default)]
pub struct FileReaderRegistry {
    readers: Vec<Box<dyn FileReader>>,
}

impl FileReaderRegistry {
    pub fn new() -> Self {
        Self { readers: Vec::new() }
    }

    pub fn register(&mut self, reader: Box<dyn FileReader>) {
        tracing::debug!("Registered file reader: {}", reader.name());
        self.readers.push(reader);
    }

    /// Find a reader that can handle this extension (case-insensitive)
    pub fn reader_for(&self, path: &Path) -> Option<&dyn FileReader> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e.to_lowercase()))
            .unwrap_or_default();

        self.readers
            .iter()
            .find(|r| r.supported_extensions().iter().any(|sup| sup.eq_ignore_ascii_case(&ext)))
            .map(|b| b.as_ref())
    }

    pub fn supported_extensions(&self) -> Vec<&'static str> {
        self.readers
            .iter()
            .flat_map(|r| r.supported_extensions())
            .copied()
            .collect()
    }
}

/// The TTS provider port. Could be OpenAI, Azure, a local model, or a very talented parrot.
#[async_trait]
pub trait TtsProvider: Send + Sync {
    /// Synthesize a single chunk into MP3 bytes.
    /// The tattler promises not to send more than ~4000 chars at once.
    async fn synthesize(&self, chunk: &TextChunk, options: &TtsOptions) -> Result<Vec<u8>>;

    /// Name of the backend for logging ("OpenAI (or your corporate tattler)")
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
}

/// Audio playback port. Plays the juicy gossip through your speakers.
/// Implementations: rodio (current), cpal direct, or even "write to /dev/null for CI".
pub trait AudioPlayer: Send + Sync {
    /// Play raw audio bytes (MP3, WAV, etc — whatever the provider gave us).
    /// Blocks until finished (or until someone hits Ctrl-C, the ultimate heckler).
    fn play(&self, audio_data: &[u8]) -> Result<()>;

    /// Optional: play multiple chunks back-to-back with nice progress.
    /// Default impl just loops play().
    fn play_chunks(&self, chunks: &[Vec<u8>]) -> Result<()> {
        for (i, chunk) in chunks.iter().enumerate() {
            tracing::debug!("Playing audio chunk {}/{}", i + 1, chunks.len());
            self.play(chunk)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn name(&self) -> &'static str;
}

/// The main application service / use-case orchestrator.
/// This is where the magic (gossip) happens. Everything else is just adapters.
#[async_trait]
pub trait TextToSpeechService {
    /// The big one: take a document, turn it into beautiful spoken word.
    async fn tattle(
        &self,
        document: ProcessedDocument,
        options: TtsOptions,
        output_path: Option<std::path::PathBuf>,
        play: bool,
    ) -> Result<TtsResult>;
}
