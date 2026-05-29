//! Placeholder for Microsoft Word (.docx) support.
//!
//! When you're ready to make the tattler read Word docs:
//! 1. Add `docx = "0.1"` (or similar) to Cargo.toml
//! 2. Implement `FileReader` for `DocxReader`
//! 3. Register it in file_readers/mod.rs
//!
//! The core use-case and CLI will never know. That's the beauty of ports & adapters.

use crate::domain::ports::FileReader;
use anyhow::Result;
use std::path::Path;

/// The future hero of corporate gossip.
#[allow(dead_code)]
pub struct DocxReader;

impl FileReader for DocxReader {
    fn name(&self) -> &'static str {
        "Word Document Tattler (not yet implemented)"
    }

    fn supported_extensions(&self) -> &'static [&'static str] {
        &[".docx"]
    }

    fn read(&self, _path: &Path) -> Result<String> {
        anyhow::bail!(
            "DOCX support is not implemented yet. \
             The tattler is still learning how to read between the (Word) lines. \
             For now, save as .txt or .md — we won't judge."
        )
    }
}
