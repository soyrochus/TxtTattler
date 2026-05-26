//! Adapters module — thin glue between domain ports and concrete infrastructure.
//! In a bigger app this might contain factory functions and DI containers.
//! For TxtTattler v1 it's mostly a nice place to keep the wiring logic out of main.

#[allow(unused_imports)]
pub use crate::infrastructure::cache::Mp3Cache;
pub use crate::infrastructure::file_readers::FILE_READER_REGISTRY;
