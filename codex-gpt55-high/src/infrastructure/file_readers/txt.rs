use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use encoding_rs::{UTF_8, WINDOWS_1252};

use crate::domain::ports::FileReader;

pub struct TxtFileReader;

impl FileReader for TxtFileReader {
    fn read(&self, path: &Path) -> Result<String> {
        let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
        Ok(decode_text_file(&bytes))
    }
}

pub fn decode_text_file(bytes: &[u8]) -> String {
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    let (decoded, _, had_errors) = UTF_8.decode(bytes);
    if !had_errors {
        return decoded.into_owned();
    }

    let (decoded, _, _) = WINDOWS_1252.decode(bytes);
    decoded.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_utf8_bom() {
        assert_eq!(decode_text_file(b"\xEF\xBB\xBFhello"), "hello");
    }
}
