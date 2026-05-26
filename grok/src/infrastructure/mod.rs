//! Infrastructure layer — concrete implementations of the domain ports.
//! Adapters that talk to the real world (files, OpenAI, speakers).

pub mod audio;
pub mod cache;
pub mod file_readers;
pub mod tts;
