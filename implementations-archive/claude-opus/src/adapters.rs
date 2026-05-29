//! The wiring layer. Right now it holds the `FileReaderRegistry` — the single
//! place that maps a file extension to the adapter that knows how to read it.
//!
//! Adding `.docx` or `.pdf` support later is a one-liner here: implement the
//! `FileReader`, then `register("docx", ...)`. The use-case and CLI don't change.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::domain::ports::FileReader;
use crate::infrastructure::file_readers::{docx::DocxReader, pdf::PdfReader, txt::TxtReader};

/// Maps lower-cased file extensions to the reader that handles them.
pub struct FileReaderRegistry {
    readers: HashMap<String, Arc<dyn FileReader>>,
}

impl FileReaderRegistry {
    /// A registry pre-loaded with every built-in reader. The `.txt` reader is
    /// fully functional; `.docx`/`.pdf` are registered but politely report
    /// "coming soon" — so the plumbing is proven end-to-end today.
    pub fn with_builtins() -> Self {
        let mut registry = FileReaderRegistry {
            readers: HashMap::new(),
        };
        registry.register("txt", Arc::new(TxtReader));
        registry.register("docx", Arc::new(DocxReader));
        registry.register("pdf", Arc::new(PdfReader));
        registry
    }

    /// Register (or replace) the reader for a given extension.
    pub fn register(&mut self, ext: &str, reader: Arc<dyn FileReader>) {
        self.readers.insert(ext.trim().to_lowercase(), reader);
    }

    /// Find the reader for a path based on its extension.
    pub fn reader_for(&self, path: &Path) -> anyhow::Result<Arc<dyn FileReader>> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_lowercase)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "'{}' has no file extension, so I can't tell what kind of file it is",
                    path.display()
                )
            })?;

        self.readers.get(&ext).cloned().ok_or_else(|| {
            anyhow::anyhow!(
                "I don't know how to read '.{ext}' files yet (known: {})",
                self.known_extensions().join(", ")
            )
        })
    }

    /// Extensions this registry can currently handle, sorted for stable output.
    pub fn known_extensions(&self) -> Vec<String> {
        let mut exts: Vec<String> = self.readers.keys().cloned().collect();
        exts.sort();
        exts
    }
}

impl Default for FileReaderRegistry {
    fn default() -> Self {
        Self::with_builtins()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn resolves_txt_reader_regardless_of_case() {
        let registry = FileReaderRegistry::with_builtins();
        assert!(registry.reader_for(&PathBuf::from("notes.TXT")).is_ok());
    }

    #[test]
    fn unknown_extension_is_rejected() {
        let registry = FileReaderRegistry::with_builtins();
        let err = registry
            .reader_for(&PathBuf::from("song.flac"))
            .err()
            .expect("unknown extension should be rejected");
        assert!(err.to_string().contains("flac"));
    }

    #[test]
    fn missing_extension_is_rejected() {
        let registry = FileReaderRegistry::with_builtins();
        assert!(registry.reader_for(&PathBuf::from("README")).is_err());
    }
}
