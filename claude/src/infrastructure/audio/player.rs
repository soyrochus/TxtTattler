use std::io::Cursor;

use rodio::{Decoder, OutputStream, Sink};

use crate::domain::ports::AudioPlayer;

/// Audio player backed by rodio.
pub struct RodioPlayer;

impl AudioPlayer for RodioPlayer {
    fn play(&self, audio_data: &[u8]) -> anyhow::Result<()> {
        let (_stream, stream_handle) =
            OutputStream::try_default().map_err(|e| anyhow::anyhow!("Audio stream error: {}", e))?;

        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| anyhow::anyhow!("Failed to create audio sink: {}", e))?;

        let cursor = Cursor::new(audio_data.to_vec());
        let decoder = Decoder::new(cursor)
            .map_err(|e| anyhow::anyhow!("Failed to decode audio: {}", e))?;

        sink.append(decoder);
        sink.sleep_until_end();

        Ok(())
    }
}
