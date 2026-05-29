use std::path::Path;

use anyhow::{Result, bail};

use crate::domain::ports::FileReader;

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct DocxFileReader;

impl FileReader for DocxFileReader {
    fn read(&self, _path: &Path) -> Result<String> {
        bail!("DOCX support is not wired in yet. The adapter stub is ready for future gossip.");
    }
}
