//! Placeholder for PDF text extraction.
//!
//! Future work: use lopdf, pdf-extract, or pdfium-render + OCR fallback for image PDFs.
//! The architecture is ready. The tattler is patient.

use crate::domain::ports::FileReader;
use anyhow::Result;
use std::path::Path;

#[allow(dead_code)]
pub struct PdfReader;

impl FileReader for PdfReader {
    fn name(&self) -> &'static str {
        "PDF Tattler (coming soon)"
    }

    fn supported_extensions(&self) -> &'static [&'static str] {
        &[".pdf"]
    }

    fn read(&self, _path: &Path) -> Result<String> {
        anyhow::bail!(
            "PDF support is still on the drawing board. \
             The tattler can currently only spill the beans on plain text files. \
             Try converting the PDF to .txt first — your secrets will still be safe with us."
        )
    }
}
