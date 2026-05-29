use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::ports::FileReader;

pub mod docx;
pub mod pdf;
pub mod txt;

#[derive(Default)]
pub struct FileReaderRegistry {
    readers: HashMap<String, Arc<dyn FileReader>>,
}

impl FileReaderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<R>(&mut self, extension: impl Into<String>, reader: R)
    where
        R: FileReader + 'static,
    {
        let normalized = normalize_extension(&extension.into());
        self.readers.insert(normalized, Arc::new(reader));
    }

    pub fn reader_for(&self, extension: &str) -> Option<Arc<dyn FileReader>> {
        self.readers.get(&normalize_extension(extension)).cloned()
    }

    pub fn default_txt_only() -> Self {
        let mut registry = Self::new();
        registry.register("txt", txt::TxtFileReader);
        registry
    }
}

fn normalize_extension(extension: &str) -> String {
    extension.trim_start_matches('.').to_ascii_lowercase()
}
