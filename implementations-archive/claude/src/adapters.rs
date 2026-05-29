use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::domain::ports::FileReader;
use crate::infrastructure::file_readers::{docx::DocxReader, pdf::PdfReader, txt::TxtReader};

/// Registry mapping file extensions to their corresponding FileReader implementations.
pub struct FileReaderRegistry {
    readers: HashMap<String, Arc<dyn FileReader>>,
}

impl FileReaderRegistry {
    /// Create a new registry pre-populated with built-in readers.
    pub fn new() -> Self {
        let mut registry = FileReaderRegistry {
            readers: HashMap::new(),
        };
        registry.register("txt", Arc::new(TxtReader));
        registry.register("docx", Arc::new(DocxReader));
        registry.register("pdf", Arc::new(PdfReader));
        registry
    }

    /// Register a reader for a given file extension.
    pub fn register(&mut self, ext: &str, reader: Arc<dyn FileReader>) {
        self.readers.insert(ext.to_lowercase(), reader);
    }

    /// Get the reader for the given file path based on its extension.
    pub fn get_for_path(&self, path: &Path) -> anyhow::Result<Arc<dyn FileReader>> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Cannot determine file type for '{}': no extension",
                    path.display()
                )
            })?;

        self.readers
            .get(&ext)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No reader registered for '.{}' files", ext))
    }
}

impl Default for FileReaderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
