use std::fs;
use std::path::Path;

use crate::domain::ports::FileReader;

/// Reads plain-text files, stripping UTF-8 BOM if present.
pub struct TxtReader;

impl FileReader for TxtReader {
    fn read(&self, path: &Path) -> anyhow::Result<String> {
        let bytes = fs::read(path)
            .map_err(|e| anyhow::anyhow!("Cannot read '{}': {}", path.display(), e))?;

        // Strip UTF-8 BOM (EF BB BF) if present
        let bytes = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            &bytes[3..]
        } else {
            &bytes[..]
        };

        Ok(String::from_utf8_lossy(bytes).into_owned())
    }
}
