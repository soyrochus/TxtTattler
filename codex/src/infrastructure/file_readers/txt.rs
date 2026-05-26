use std::{borrow::Cow, fs, path::Path};

use anyhow::{Context, Result};
use encoding_rs::UTF_8;

use crate::domain::ports::FileReader;

pub struct TxtFileReader;

impl FileReader for TxtFileReader {
    fn read(&self, path: &Path) -> Result<String> {
        let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
        let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(&bytes);
        let (decoded, _, had_errors) = UTF_8.decode(bytes);

        if had_errors {
            anyhow::bail!(
                "{} is not valid UTF-8. TxtTattler v1 only tattles UTF-8 text.",
                path.display()
            );
        }

        Ok(match decoded {
            Cow::Borrowed(value) => value.to_owned(),
            Cow::Owned(value) => value,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn reads_utf8_with_bom() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(&[0xEF, 0xBB, 0xBF]).unwrap();
        file.write_all("hello".as_bytes()).unwrap();

        let text = TxtFileReader.read(file.path()).unwrap();

        assert_eq!(text, "hello");
    }
}
