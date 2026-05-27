//! Adapters that touch the outside world: the filesystem, OpenAI, the speakers.
//! Everything dirty lives here, behind the clean ports defined in `domain`.

pub mod audio;
pub mod file_readers;
pub mod tts;
