//! Placeholder PDF reader. Same deal as DOCX: wire in `lopdf`/`pdf-extract`
//! later, extract the page text here, and the rest of TxtTattler won't notice.

use std::path::Path;

use crate::domain::ports::FileReader;

pub struct PdfReader;

impl FileReader for PdfReader {
    fn read(&self, _path: &Path) -> anyhow::Result<String> {
        anyhow::bail!(
            "PDF support is coming soon! For now, export your document to .txt and I'll read every juicy word."
        )
    }
}
