use std::{fs, path::Path};

use anyhow::{Context, Result, bail};
use encoding_rs::{UTF_16BE, UTF_16LE};

use crate::domain::ports::FileReader;

#[derive(Debug, Default)]
pub struct TxtFileReader;

impl FileReader for TxtFileReader {
    fn read(&self, path: &Path) -> Result<String> {
        let bytes = fs::read(path)
            .with_context(|| format!("Failed to read text file '{}'.", path.display()))?;

        if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            return String::from_utf8(bytes[3..].to_vec())
                .with_context(|| format!("File '{}' is not valid UTF-8.", path.display()));
        }

        if bytes.starts_with(&[0xFF, 0xFE]) {
            let (decoded, _, had_errors) = UTF_16LE.decode(&bytes[2..]);
            if had_errors {
                bail!("File '{}' contains invalid UTF-16LE data.", path.display());
            }
            return Ok(decoded.into_owned());
        }

        if bytes.starts_with(&[0xFE, 0xFF]) {
            let (decoded, _, had_errors) = UTF_16BE.decode(&bytes[2..]);
            if had_errors {
                bail!("File '{}' contains invalid UTF-16BE data.", path.display());
            }
            return Ok(decoded.into_owned());
        }

        String::from_utf8(bytes).with_context(|| {
            format!(
                "File '{}' must be UTF-8 or BOM-prefixed UTF-16.",
                path.display()
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::TxtFileReader;
    use crate::domain::ports::FileReader;
    use std::{fs, io::Write};
    use tempfile::NamedTempFile;

    #[test]
    fn reader_supports_utf8_bom() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&[0xEF, 0xBB, 0xBF]).unwrap();
        file.write_all(b"hello").unwrap();

        let reader = TxtFileReader;
        let text = reader.read(file.path()).unwrap();
        assert_eq!(text, "hello");
    }

    #[test]
    fn reader_supports_plain_utf8() {
        let file = NamedTempFile::new().unwrap();
        fs::write(file.path(), "hello world").unwrap();

        let reader = TxtFileReader;
        let text = reader.read(file.path()).unwrap();
        assert_eq!(text, "hello world");
    }

    #[test]
    fn reader_supports_utf16le_bom() {
        let file = NamedTempFile::new().unwrap();
        fs::write(file.path(), [0xFF, 0xFE, b'h', 0x00, b'i', 0x00]).unwrap();

        let reader = TxtFileReader;
        let text = reader.read(file.path()).unwrap();
        assert_eq!(text, "hi");
    }

    #[test]
    fn reader_supports_utf16be_bom() {
        let file = NamedTempFile::new().unwrap();
        fs::write(file.path(), [0xFE, 0xFF, 0x00, b'h', 0x00, b'i']).unwrap();

        let reader = TxtFileReader;
        let text = reader.read(file.path()).unwrap();
        assert_eq!(text, "hi");
    }

    #[test]
    fn reader_rejects_invalid_encoding() {
        let file = NamedTempFile::new().unwrap();
        fs::write(file.path(), [0xFF, 0xFE, 0x00]).unwrap();

        let reader = TxtFileReader;
        let err = reader.read(file.path()).unwrap_err();
        assert!(err.to_string().contains("invalid UTF-16LE"));
    }
}
