//! File reader adapters.
//! Today: only .txt (the humble beginning of a beautiful gossip empire).
//! Tomorrow: .docx, .pdf, .md, .rtf... the tattler has big plans.

pub mod docx;
pub mod pdf;
pub mod txt;

use once_cell::sync::Lazy;
use std::sync::Arc;

/// The global registry of all file gossips we understand.
// Using Lazy so registration can happen at startup in main.
pub static FILE_READER_REGISTRY: Lazy<Arc<crate::domain::ports::FileReaderRegistry>> = Lazy::new(|| {
    let mut reg = crate::domain::ports::FileReaderRegistry::new();
    reg.register(Box::new(txt::TxtReader));
    // Future readers will be registered here — zero changes to use case or CLI!
    // reg.register(Box::new(docx::DocxReader));
    // reg.register(Box::new(pdf::PdfReader));
    Arc::new(reg)
});

#[allow(unused_imports)]
pub use docx::DocxReader;
#[allow(unused_imports)]
pub use pdf::PdfReader;
#[allow(unused_imports)]
pub use txt::TxtReader;
