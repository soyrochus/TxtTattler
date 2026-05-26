use std::path::Path;

use crate::domain::ports::FileReader;

/// Placeholder PDF reader — not yet implemented.
pub struct PdfReader;

impl FileReader for PdfReader {
    fn read(&self, _path: &Path) -> anyhow::Result<String> {
        Err(anyhow::anyhow!("PDF support coming soon..."))
    }
}
