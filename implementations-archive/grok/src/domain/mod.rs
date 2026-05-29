//! Domain layer for TxtTattler.
//! Pure business concepts — no I/O, no frameworks, just the rules of the gossip.
//!
//! "A tattler never reveals its sources... but it will read every word you give it."

pub mod entities;
pub mod ports;

// Re-export the most important types for convenience inside the crate.
// The explicit list (instead of globs) is intentional for discoverability.
#[allow(unused_imports)]
pub use entities::{ProcessedDocument, TextChunk, TtsOptions, TtsResult, Voice, TtsModel};
#[allow(unused_imports)]
pub use ports::{AudioPlayer, FileReader, FileReaderRegistry, TextToSpeechService, TtsProvider};
