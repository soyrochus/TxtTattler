use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    process,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use tracing_subscriber::EnvFilter;

use crate::domain::ports::{ProgressHandle, Reporter};

pub fn init_tracing(verbose: bool) {
    let filter = if verbose { "debug" } else { "info" };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(filter))
        .with_target(false)
        .without_time()
        .try_init();
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("txttattler-output");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temp_path = parent.join(format!(".{filename}.{}.{}.tmp", process::id(), nonce));

    let write_result = (|| -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .with_context(|| format!("Failed to create temporary file {}", temp_path.display()))?;
        file.write_all(bytes)
            .with_context(|| format!("Failed to write temporary file {}", temp_path.display()))?;
        file.flush()
            .with_context(|| format!("Failed to flush temporary file {}", temp_path.display()))?;
        file.sync_all()
            .with_context(|| format!("Failed to sync temporary file {}", temp_path.display()))?;
        fs::rename(&temp_path, path).with_context(|| {
            format!(
                "Failed to rename temporary file {} to {}",
                temp_path.display(),
                path.display()
            )
        })?;
        Ok(())
    })();

    if write_result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }

    write_result
}

#[derive(Clone)]
pub struct ConsoleReporter {
    verbose: bool,
}

#[derive(Clone, Default)]
pub struct SilentReporter;

impl Reporter for SilentReporter {
    fn status(&self, _message: &str) {}

    fn detail(&self, _message: &str) {}

    fn success(&self, _message: &str) {}

    fn progress(&self, _message: &str, _len: u64) -> Box<dyn ProgressHandle> {
        Box::new(SilentProgressHandle)
    }
}

#[derive(Clone, Default)]
struct SilentProgressHandle;

impl ProgressHandle for SilentProgressHandle {
    fn inc(&self, _delta: u64) {}

    fn set_message(&self, _message: &str) {}

    fn finish_with_message(&self, _message: &str) {}
}

#[cfg(test)]
mod tests {
    use super::atomic_write;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn atomic_write_replaces_existing_file() {
        let tempdir = TempDir::new().unwrap();
        let path = tempdir.path().join("out.mp3");
        fs::write(&path, b"old").unwrap();

        atomic_write(&path, b"new").unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"new");
    }
}

impl ConsoleReporter {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}

impl Reporter for ConsoleReporter {
    fn status(&self, message: &str) {
        println!("{} {}", style("🎙").cyan(), style(message).bold());
    }

    fn detail(&self, message: &str) {
        if self.verbose {
            println!("{} {}", style("·").dim(), style(message).dim());
        }
    }

    fn success(&self, message: &str) {
        println!("{} {}", style("✨").green(), style(message).green().bold());
    }

    fn progress(&self, message: &str, len: u64) -> Box<dyn ProgressHandle> {
        let progress = ProgressBar::new(len);
        let style = ProgressStyle::with_template(
            "{spinner:.cyan} {msg} [{bar:30.magenta/blue}] {pos}/{len}",
        )
        .expect("progress template is valid")
        .progress_chars("=>-");
        progress.set_style(style);
        progress.set_message(message.to_string());
        Box::new(IndicatifProgressHandle {
            inner: Arc::new(progress),
        })
    }
}

struct IndicatifProgressHandle {
    inner: Arc<ProgressBar>,
}

impl ProgressHandle for IndicatifProgressHandle {
    fn inc(&self, delta: u64) {
        self.inner.inc(delta);
    }

    fn set_message(&self, message: &str) {
        self.inner.set_message(message.to_string());
    }

    fn finish_with_message(&self, message: &str) {
        self.inner.finish_with_message(message.to_string());
    }
}
