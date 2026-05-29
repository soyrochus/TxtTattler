//! Plain-text reader. UTF-8 with optional BOM; invalid bytes are replaced
//! rather than erroring, because the tattler would rather mumble than refuse.

use std::fs;
use std::path::Path;

use anyhow::Context;

use crate::domain::ports::FileReader;

pub struct TxtReader;

impl FileReader for TxtReader {
    fn read(&self, path: &Path) -> anyhow::Result<String> {
        let bytes = fs::read(path)
            .with_context(|| format!("couldn't open '{}'", path.display()))?;

        // Strip a UTF-8 BOM (EF BB BF) if the file has one.
        let bytes = bytes
            .strip_prefix(&[0xEF, 0xBB, 0xBF])
            .unwrap_or(&bytes);

        Ok(String::from_utf8_lossy(bytes).into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn reads_plain_utf8() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        write!(file, "hello there").unwrap();
        let text = TxtReader.read(file.path()).unwrap();
        assert_eq!(text, "hello there");
    }

    #[test]
    fn strips_utf8_bom() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(&[0xEF, 0xBB, 0xBF]).unwrap();
        file.write_all(b"bom be gone").unwrap();
        let text = TxtReader.read(file.path()).unwrap();
        assert_eq!(text, "bom be gone");
    }

    #[test]
    fn missing_file_errors() {
        assert!(TxtReader.read(Path::new("/no/such/file.txt")).is_err());
    }
}
