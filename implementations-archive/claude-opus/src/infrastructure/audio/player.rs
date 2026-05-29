//! rodio-backed MP3 playback. Cross-platform (ALSA/PulseAudio, CoreAudio,
//! WASAPI) with zero OS-specific code leaking outside this file.

use std::io::Cursor;

use anyhow::Context;
use rodio::{Decoder, OutputStream, Sink};

use crate::domain::ports::AudioPlayer;

pub struct RodioPlayer;

impl AudioPlayer for RodioPlayer {
    fn play(&self, audio: &[u8]) -> anyhow::Result<()> {
        let (_stream, handle) = OutputStream::try_default()
            .context("no audio output device available (is a sound server running?)")?;
        let sink = Sink::try_new(&handle).context("failed to create audio sink")?;

        // Clone into an owned cursor so the decoder doesn't borrow our slice.
        let decoder = Decoder::new(Cursor::new(audio.to_vec()))
            .context("failed to decode MP3 audio")?;

        sink.append(decoder);
        sink.sleep_until_end();
        Ok(())
    }
}
