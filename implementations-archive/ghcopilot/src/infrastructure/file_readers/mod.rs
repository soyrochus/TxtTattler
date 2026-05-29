use std::{collections::HashMap, path::Path, sync::Arc};

use anyhow::{Result, anyhow, bail};

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

    pub fn register(&mut self, extension: &str, reader: Arc<dyn FileReader>) {
        self.readers.insert(
            extension.trim_start_matches('.').to_ascii_lowercase(),
            reader,
        );
    }

    pub fn read(&self, path: &Path) -> Result<String> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase)
            .ok_or_else(|| anyhow!("File '{}' has no extension.", path.display()))?;

        let reader = self.readers.get(&extension).ok_or_else(|| {
            let supported = self.supported_extensions().join(", ");
            anyhow!(
                "Unsupported extension '.{}'. Supported extensions: {}",
                extension,
                supported
            )
        })?;

        if !path.exists() {
            bail!("Input file '{}' does not exist.", path.display());
        }

        reader.read(path)
    }

    pub fn supported_extensions(&self) -> Vec<String> {
        let mut extensions = self.readers.keys().cloned().collect::<Vec<_>>();
        extensions.sort();
        extensions
    }
}

#[cfg(test)]
mod tests {
    use super::FileReaderRegistry;
    use crate::{domain::ports::FileReader, infrastructure::file_readers::txt::TxtFileReader};
    use std::sync::Arc;

    #[test]
    fn registry_reports_supported_extensions() {
        let mut registry = FileReaderRegistry::new();
        registry.register(
            "txt",
            Arc::new(TxtFileReader::default()) as Arc<dyn FileReader>,
        );
        assert_eq!(registry.supported_extensions(), vec!["txt".to_string()]);
    }
}
