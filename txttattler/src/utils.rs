use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    process,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use console::{Term, style};
use indicatif::{ProgressBar, ProgressStyle};
use tracing_subscriber::EnvFilter;

use crate::domain::ports::{
    PlaybackProgressHandle, ProgressHandle, Reporter, clamp_playback_elapsed, format_playback_time,
};

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

    fn playback(&self, _total: Option<Duration>) -> Box<dyn PlaybackProgressHandle> {
        Box::new(SilentPlaybackProgressHandle)
    }
}

#[derive(Clone, Default)]
struct SilentProgressHandle;

impl ProgressHandle for SilentProgressHandle {
    fn inc(&self, _delta: u64) {}

    fn set_message(&self, _message: &str) {}

    fn finish_with_message(&self, _message: &str) {}
}

#[derive(Clone, Default)]
struct SilentPlaybackProgressHandle;

impl PlaybackProgressHandle for SilentPlaybackProgressHandle {
    fn tick(&self, _elapsed: Duration) {}

    fn finish(&self) {}
}

#[cfg(test)]
mod tests {
    use super::{StaticPlaybackProgressHandle, atomic_write};
    use crate::domain::ports::PlaybackProgressHandle;
    use std::fs;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn atomic_write_replaces_existing_file() {
        let tempdir = TempDir::new().unwrap();
        let path = tempdir.path().join("out.mp3");
        fs::write(&path, b"old").unwrap();

        atomic_write(&path, b"new").unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"new");
    }

    #[test]
    fn static_playback_progress_accepts_unknown_duration() {
        let handle = StaticPlaybackProgressHandle { total: None };
        handle.tick(Duration::from_secs(3));
        handle.finish();
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

    fn playback(&self, total: Option<Duration>) -> Box<dyn PlaybackProgressHandle> {
        if let Some(total) = total
            && Term::stdout().is_term()
        {
            let total_ms = duration_millis_u64(total);
            let progress = ProgressBar::new(total_ms);
            let style = ProgressStyle::with_template(
                "{spinner:.cyan} {msg} [{bar:30.green/blue}] {percent:>3}%",
            )
            .expect("playback progress template is valid")
            .progress_chars("=>-");
            progress.set_style(style);
            progress.set_message(format!("Playing 00:00 / {}", format_playback_time(total)));
            return Box::new(IndicatifPlaybackProgressHandle {
                inner: Arc::new(progress),
                total,
            });
        }

        match total {
            Some(total) => println!(
                "{} {}",
                style("▶").cyan(),
                style(format!(
                    "Playback started ({}).",
                    format_playback_time(total)
                ))
                .bold()
            ),
            None => println!(
                "{} {}",
                style("▶").cyan(),
                style("Playback started. Duration unknown.").bold()
            ),
        }
        Box::new(StaticPlaybackProgressHandle { total })
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

struct IndicatifPlaybackProgressHandle {
    inner: Arc<ProgressBar>,
    total: Duration,
}

impl PlaybackProgressHandle for IndicatifPlaybackProgressHandle {
    fn tick(&self, elapsed: Duration) {
        let elapsed = clamp_playback_elapsed(elapsed, self.total);
        self.inner.set_position(duration_millis_u64(elapsed));
        self.inner.set_message(format!(
            "Playing {} / {}",
            format_playback_time(elapsed),
            format_playback_time(self.total)
        ));
    }

    fn finish(&self) {
        self.inner.set_position(duration_millis_u64(self.total));
        self.inner.finish_with_message(format!(
            "Playback finished ({})",
            format_playback_time(self.total)
        ));
    }
}

struct StaticPlaybackProgressHandle {
    total: Option<Duration>,
}

impl PlaybackProgressHandle for StaticPlaybackProgressHandle {
    fn tick(&self, _elapsed: Duration) {}

    fn finish(&self) {
        match self.total {
            Some(total) => println!(
                "{} {}",
                style("✓").green(),
                style(format!(
                    "Playback finished ({}).",
                    format_playback_time(total)
                ))
                .green()
                .bold()
            ),
            None => println!(
                "{} {}",
                style("✓").green(),
                style("Playback finished.").green().bold()
            ),
        }
    }
}

fn duration_millis_u64(duration: Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}
