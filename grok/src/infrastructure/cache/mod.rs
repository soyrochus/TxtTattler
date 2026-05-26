//! MP3 caching infrastructure for TxtTattler.
//!
//! The tattler is thrifty — it hates repeating itself (and wasting your OpenAI credits).
//! We cache the final synthesized MP3 using a content + parameter hash so that
//! identical runs are instant and free.

pub mod mp3_cache;

pub use mp3_cache::Mp3Cache;
