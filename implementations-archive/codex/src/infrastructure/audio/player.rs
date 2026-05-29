use std::io::Cursor;

use anyhow::{Context, Result};
use rodio::{Decoder, OutputStream, Sink};

use crate::domain::ports::AudioPlayer;

pub struct RodioAudioPlayer;

impl AudioPlayer for RodioAudioPlayer {
    fn play(&self, mp3_bytes: &[u8]) -> Result<()> {
        let cursor = Cursor::new(mp3_bytes.to_vec());
        let source = Decoder::new(cursor).context("failed to decode generated MP3")?;
        let (_stream, handle) =
            OutputStream::try_default().context("failed to open default audio output")?;
        let sink = Sink::try_new(&handle).context("failed to create audio sink")?;

        sink.append(source);
        sink.sleep_until_end();
        Ok(())
    }
}
