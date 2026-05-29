//! TTS provider adapters.
//! Currently only the mighty OpenAI (and its corporate twin Azure).

pub mod openai;

pub use openai::OpenAiTtsProvider;
