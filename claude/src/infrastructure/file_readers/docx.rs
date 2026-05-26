use std::path::Path;

use crate::domain::ports::FileReader;

/// Placeholder DOCX reader — not yet implemented.
pub struct DocxReader;

impl FileReader for DocxReader {
    fn read(&self, _path: &Path) -> anyhow::Result<String> {
        Err(anyhow::anyhow!("DOCX support coming soon..."))
    }
}
