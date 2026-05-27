use std::{
    io::Cursor,
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use rodio::{Decoder, DeviceSinkBuilder, Player, Source};

use crate::domain::ports::{AudioPlayer, Reporter};

#[derive(Debug, Default)]
pub struct RodioAudioPlayer;

impl AudioPlayer for RodioAudioPlayer {
    fn play_mp3(&self, audio: Vec<u8>, reporter: &dyn Reporter) -> Result<()> {
        let device = DeviceSinkBuilder::open_default_sink().context(
            "Failed to open the default audio output device for generated audio playback.",
        )?;
        let player = Player::connect_new(&device.mixer());
        let source =
            decode_mp3(audio).context("Failed to decode generated MP3 audio for playback.")?;
        let total_duration = source.total_duration();
        let progress = reporter.playback(total_duration);

        player.append(source);
        track_playback_until_end(&player, progress.as_ref(), total_duration);

        drop(player);
        drop(device);
        Ok(())
    }
}

fn decode_mp3(audio: Vec<u8>) -> Result<Decoder<Cursor<Vec<u8>>>> {
    let byte_len = audio.len() as u64;
    Decoder::builder()
        .with_data(Cursor::new(audio))
        .with_byte_len(byte_len)
        .build()
        .context("Failed to create MP3 decoder")
}

fn track_playback_until_end(
    player: &Player,
    progress: &dyn crate::domain::ports::PlaybackProgressHandle,
    total_duration: Option<Duration>,
) {
    let started = Instant::now();
    while !player.empty() {
        let elapsed = total_duration
            .map(|total| started.elapsed().min(total))
            .unwrap_or_else(|| started.elapsed());
        progress.tick(elapsed);
        thread::sleep(Duration::from_millis(150));
    }
    progress.finish();
}
