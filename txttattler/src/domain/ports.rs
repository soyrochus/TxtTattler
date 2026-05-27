use std::{path::Path, time::Duration};

use anyhow::Result;
use async_trait::async_trait;

use crate::domain::entities::TtsRequest;

pub trait FileReader: Send + Sync {
    fn read(&self, path: &Path) -> Result<String>;
}

pub trait DocumentReader: Send + Sync {
    fn read(&self, path: &Path) -> Result<String>;
}

#[async_trait]
pub trait TtsProvider: Send + Sync {
    async fn synthesize(&self, request: &TtsRequest) -> Result<Vec<u8>>;
}

pub trait AudioPlayer: Send + Sync {
    fn play_mp3(&self, audio: Vec<u8>, reporter: &dyn Reporter) -> Result<()>;
}

pub trait TextProcessor: Send + Sync {
    fn name(&self) -> &'static str;
    fn process(&self, input: String) -> Result<String>;
}

pub trait ProgressHandle: Send {
    fn inc(&self, delta: u64);
    fn set_message(&self, message: &str);
    fn finish_with_message(&self, message: &str);
}

pub trait PlaybackProgressHandle: Send {
    fn tick(&self, elapsed: Duration);
    fn finish(&self);
}

pub trait Reporter: Send + Sync {
    fn status(&self, message: &str);
    fn detail(&self, message: &str);
    fn success(&self, message: &str);
    fn progress(&self, message: &str, len: u64) -> Box<dyn ProgressHandle>;
    fn playback(&self, total: Option<Duration>) -> Box<dyn PlaybackProgressHandle>;
}

pub fn clamp_playback_elapsed(elapsed: Duration, total: Duration) -> Duration {
    elapsed.min(total)
}

pub fn format_playback_time(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let seconds = total_seconds % 60;
    let minutes = (total_seconds / 60) % 60;
    let hours = total_seconds / 3_600;

    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::{clamp_playback_elapsed, format_playback_time};
    use std::time::Duration;

    #[test]
    fn playback_time_uses_minutes_and_seconds_under_an_hour() {
        assert_eq!(format_playback_time(Duration::from_secs(65)), "01:05");
    }

    #[test]
    fn playback_time_uses_hours_when_needed() {
        assert_eq!(format_playback_time(Duration::from_secs(3_665)), "01:01:05");
    }

    #[test]
    fn playback_elapsed_is_clamped_to_total() {
        assert_eq!(
            clamp_playback_elapsed(Duration::from_secs(20), Duration::from_secs(5)),
            Duration::from_secs(5)
        );
    }
}
