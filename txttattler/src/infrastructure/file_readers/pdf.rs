use std::path::Path;

use anyhow::{Result, bail};

use crate::domain::ports::FileReader;

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct PdfFileReader;

impl FileReader for PdfFileReader {
    fn read(&self, _path: &Path) -> Result<String> {
        bail!("PDF support is still a future scandal. The placeholder adapter is in place.");
    }
}
