//! Placeholder DOCX reader. The slot exists; the gossip just can't read Word
//! files *yet*. When you're ready: add the `docx`/`docx-rs` crate, pull the
//! document body text out here, and you're done — nothing else has to change.

use std::path::Path;

use crate::domain::ports::FileReader;

pub struct DocxReader;

impl FileReader for DocxReader {
    fn read(&self, _path: &Path) -> anyhow::Result<String> {
        anyhow::bail!(
            "DOCX support is coming soon! For now, export your document to .txt and I'll spill the tea."
        )
    }
}
