//! The original tattler. Plain text files only.
//! UTF-8 with optional BOM support because the world is messy and gossips must be robust.

use crate::domain::ports::FileReader;
use anyhow::{Context, Result};
use encoding_rs::UTF_8;
use std::fs;
use std::path::Path;

/// The mighty .txt reader. Simple, fast, and surprisingly good at keeping secrets until you ask.
pub struct TxtReader;

impl FileReader for TxtReader {
    fn name(&self) -> &'static str {
        "Plain Text Tattler"
    }

    fn supported_extensions(&self) -> &'static [&'static str] {
        &[".txt", ".text", ".md", ".markdown"] // .md is a nice bonus for v1
    }

    fn read(&self, path: &Path) -> Result<String> {
        let bytes = fs::read(path)
            .with_context(|| format!("Failed to open the gossip file at {}", path.display()))?;

        // Handle UTF-8 BOM if present (some editors are dramatic like that)
        let (cow, had_errors) = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            UTF_8.decode_without_bom_handling(&bytes[3..])
        } else {
            UTF_8.decode_without_bom_handling(&bytes)
        };
        let _encoding_used = UTF_8; // we always force UTF-8 handling

        if had_errors {
            tracing::warn!(
                "The file {} contained invalid UTF-8. Some characters were replaced with �. \
                 The tattler did its best but some secrets may be garbled.",
                path.display()
            );
        }

        // encoding_used here is always UTF_8 because we used decode_without_bom_handling.
        // Real detection would use `encoding_rs::Encoding::for_label` on the bytes.
        if had_errors {
            tracing::warn!(
                "File {} had encoding issues during UTF-8 decode. Some characters may be �. The tattler tried its best.",
                path.display()
            );
        }

        let text = cow.into_owned();

        // Gentle post-processing: normalize weird whitespace, strip excessive blank lines.
        // This is the "middleware" the prompt asked for — easily injectable later.
        let cleaned = clean_text(&text);

        tracing::debug!(
            "TxtReader slurped {} chars ({} words) from {}",
            cleaned.len(),
            cleaned.split_whitespace().count(),
            path.display()
        );

        Ok(cleaned)
    }
}

/// Text cleaning pipeline. The tattler likes things neat before it starts blabbing.
fn clean_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());

    // Collapse 3+ newlines into 2 (one blank line max)
    let mut consecutive_newlines = 0;
    for ch in input.chars() {
        if ch == '\n' {
            consecutive_newlines += 1;
            if consecutive_newlines <= 2 {
                out.push('\n');
            }
        } else {
            consecutive_newlines = 0;
            out.push(ch);
        }
    }

    // Trim trailing whitespace per line, and overall
    let lines: Vec<&str> = out
        .lines()
        .map(|l| l.trim_end())
        .filter(|l| !l.is_empty() || true) // keep single blank lines
        .collect();

    // Re-join with single newlines, but keep paragraph breaks
    let mut result = String::new();
    let mut prev_blank = false;
    for line in lines {
        if line.is_empty() {
            if !prev_blank {
                result.push('\n');
            }
            prev_blank = true;
        } else {
            if !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
            }
            result.push_str(line);
            prev_blank = false;
        }
    }

    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_excessive_whitespace() {
        let messy = "Hello\n\n\n\nWorld\n\n\n   \n  \nFoo";
        let clean = clean_text(messy);
        // The tattler keeps at most one blank line between paragraphs
        assert!(clean.contains("Hello"));
        assert!(clean.contains("World"));
        assert!(clean.contains("Foo"));
        assert!(!clean.contains("\n\n\n")); // no triple newlines
    }

    #[test]
    fn handles_bom_and_trailing() {
        // We test the logic, actual BOM is handled in read()
        let text = "  Secret gossip here.  \n\n\n  ";
        assert_eq!(clean_text(text), "Secret gossip here.");
    }
}
