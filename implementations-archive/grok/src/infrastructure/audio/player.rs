//! Rodio-based audio player.
//! Plays the MP3 gossip through your speakers with minimal drama.

use crate::domain::ports::AudioPlayer;
use anyhow::{Context, Result};
use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;

/// The default cross-platform tattler voice box.
pub struct RodioAudioPlayer;

impl AudioPlayer for RodioAudioPlayer {
    fn name(&self) -> &'static str {
        "Rodio (cross-platform MP3 whisperer)"
    }

    fn play(&self, audio_data: &[u8]) -> Result<()> {
        if audio_data.is_empty() {
            return Ok(());
        }

        // Create output stream + handle. Must live until we're done playing.
        let (_stream, stream_handle) = OutputStream::try_default()
            .context("Failed to open audio output device. Is your sound working? The tattler needs ears.")?;

        let sink = Sink::try_new(&stream_handle)
            .context("Could not create audio sink. Another app hogging the speakers?")?;

        // Decode the MP3 bytes we got from OpenAI
        let cursor = Cursor::new(audio_data.to_vec());
        let decoder = Decoder::new(cursor)
            .context("Rodio could not decode the MP3 the cloud sent us. Corrupt gossip?")?;

        sink.append(decoder);
        sink.sleep_until_end(); // Block until the current juicy segment is fully tattled

        Ok(())
    }

    fn play_chunks(&self, chunks: &[Vec<u8>]) -> Result<()> {
        if chunks.is_empty() {
            return Ok(());
        }

        let (_stream, stream_handle) = OutputStream::try_default().context("No audio device available")?;
        let sink = Sink::try_new(&stream_handle).context("Failed to create audio sink")?;

        for (i, chunk) in chunks.iter().enumerate() {
            if chunk.is_empty() {
                continue;
            }
            let cursor = Cursor::new(chunk.clone());
            match Decoder::new(cursor) {
                Ok(decoder) => {
                    tracing::debug!("Queueing audio chunk {}/{} for playback", i + 1, chunks.len());
                    sink.append(decoder);
                }
                Err(e) => {
                    tracing::warn!("Skipping chunk {} — decode failed: {}", i + 1, e);
                }
            }
        }

        sink.sleep_until_end();
        Ok(())
    }
}
