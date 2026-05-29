use std::io::{BufReader, Cursor};

use anyhow::{Context, Result};
use rodio::{Decoder, DeviceSinkBuilder, Player};

use crate::domain::ports::AudioPlayer;

#[derive(Debug, Default)]
pub struct RodioAudioPlayer;

impl AudioPlayer for RodioAudioPlayer {
    fn play_mp3(&self, audio: Vec<u8>) -> Result<()> {
        let device = DeviceSinkBuilder::open_default_sink()
            .context("Failed to open the default audio output device.")?;
        let player = Player::connect_new(&device.mixer());
        let cursor = Cursor::new(audio);
        let source = Decoder::try_from(BufReader::new(cursor))
            .context("Failed to decode MP3 audio for playback.")?;
        player.append(source);
        player.sleep_until_end();
        drop(player);
        drop(device);
        Ok(())
    }
}
