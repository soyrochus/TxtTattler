use std::io::Cursor;

use anyhow::{Context, Result};
use rodio::{Decoder, OutputStreamBuilder, Sink};

use crate::domain::ports::AudioPlayer;

pub struct RodioAudioPlayer;

impl AudioPlayer for RodioAudioPlayer {
    fn play(&self, mp3_bytes: &[u8]) -> Result<()> {
        let stream = OutputStreamBuilder::open_default_stream()
            .context("no default audio output device is available")?;
        let sink = Sink::connect_new(stream.mixer());
        let source = Decoder::try_from(Cursor::new(mp3_bytes.to_vec()))
            .context("failed to decode MP3 audio")?;
        sink.append(source);
        sink.sleep_until_end();
        Ok(())
    }
}
