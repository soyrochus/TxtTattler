//! The one use-case TxtTattler has: read a file, turn it into speech, cache it,
//! optionally save it, and play it. Pure orchestration over the domain ports —
//! it reports progress through a `ProgressReporter` so it never imports a UI.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;

use crate::domain::entities::TtsOptions;
use crate::domain::ports::{AudioPlayer, FileReader, TextProcessor, TtsProvider};
use crate::utils::{MAX_CHUNK_CHARS, chunk_text, compute_cache_key, concat_mp3};

/// What the caller asks for. Paths are already resolved by the time we get here.
pub struct TtsRequest {
    pub input_path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub cache_dir: PathBuf,
    pub options: TtsOptions,
    pub play: bool,
    pub use_cache: bool,
    pub refresh: bool,
}

/// What actually happened — handy for the CLI summary and for assertions in tests.
#[derive(Debug, Clone)]
pub struct TtsOutcome {
    pub cache_hit: bool,
    pub cache_path: PathBuf,
    pub char_count: usize,
    pub chunk_count: usize,
    pub audio_len: usize,
    pub played: bool,
    pub output_path: Option<PathBuf>,
}

/// Phase-by-phase progress callbacks. Every method has a no-op default, so a
/// silent run is just an empty `impl`. The CLI supplies a colourful version.
#[allow(unused_variables)]
pub trait ProgressReporter: Send + Sync {
    fn reading(&self, path: &Path) {}
    fn text_ready(&self, chars: usize, chunks: usize) {}
    fn cache_hit(&self, path: &Path) {}
    fn synthesis_started(&self, chunks: usize) {}
    fn chunk_done(&self, done: usize, total: usize) {}
    fn synthesis_finished(&self) {}
    fn cached(&self, path: &Path) {}
    fn output_saved(&self, path: &Path) {}
    fn playing(&self) {}
    fn finished(&self, outcome: &TtsOutcome) {}
}

/// A reporter that says nothing. Useful for tests and `--no-play` automation.
pub struct SilentReporter;
impl ProgressReporter for SilentReporter {}

/// The use-case, holding its injected collaborators.
pub struct TextToSpeech {
    reader: Arc<dyn FileReader>,
    processor: Arc<dyn TextProcessor>,
    tts: Arc<dyn TtsProvider>,
    player: Arc<dyn AudioPlayer>,
}

impl TextToSpeech {
    pub fn new(
        reader: Arc<dyn FileReader>,
        processor: Arc<dyn TextProcessor>,
        tts: Arc<dyn TtsProvider>,
        player: Arc<dyn AudioPlayer>,
    ) -> Self {
        TextToSpeech {
            reader,
            processor,
            tts,
            player,
        }
    }

    pub async fn run(
        &self,
        req: TtsRequest,
        reporter: &dyn ProgressReporter,
    ) -> anyhow::Result<TtsOutcome> {
        // 1. Read + clean the text.
        reporter.reading(&req.input_path);
        let raw = self
            .reader
            .read(&req.input_path)
            .with_context(|| format!("failed to read '{}'", req.input_path.display()))?;
        let text = self.processor.process(&raw);
        if text.trim().is_empty() {
            anyhow::bail!(
                "'{}' has no readable text after cleanup — nothing to gossip about!",
                req.input_path.display()
            );
        }

        let char_count = text.chars().count();
        let chunks = chunk_text(&text, MAX_CHUNK_CHARS);
        reporter.text_ready(char_count, chunks.len());

        // 2. Work out where the cached MP3 would live.
        let key = compute_cache_key(&text, &req.options);
        let cache_path = req.cache_dir.join(key.file_name());

        // 3. Cache hit? Reuse it and skip the API entirely.
        let cache_is_fresh = req.use_cache && !req.refresh && cache_path.exists();
        let (audio, cache_hit) = if cache_is_fresh {
            reporter.cache_hit(&cache_path);
            let bytes = fs::read(&cache_path)
                .with_context(|| format!("failed to read cache '{}'", cache_path.display()))?;
            (bytes, true)
        } else {
            // 4. Synthesize chunk by chunk, then stitch the MP3 segments.
            reporter.synthesis_started(chunks.len());
            let total = chunks.len();
            let mut segments = Vec::with_capacity(total);
            for (i, chunk) in chunks.iter().enumerate() {
                let segment = self
                    .tts
                    .synthesize(chunk, &req.options)
                    .await
                    .with_context(|| format!("TTS synthesis failed on chunk {}/{total}", i + 1))?;
                segments.push(segment);
                reporter.chunk_done(i + 1, total);
            }
            reporter.synthesis_finished();
            (concat_mp3(&segments), false)
        };

        // 5. Write the cache unless disabled or already up to date.
        if req.use_cache && !cache_hit {
            fs::create_dir_all(&req.cache_dir).with_context(|| {
                format!("failed to create cache dir '{}'", req.cache_dir.display())
            })?;
            fs::write(&cache_path, &audio)
                .with_context(|| format!("failed to write cache '{}'", cache_path.display()))?;
            reporter.cached(&cache_path);
        }

        // 6. Save a user-visible copy if requested.
        if let Some(output) = &req.output_path {
            if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
                fs::create_dir_all(parent).with_context(|| {
                    format!("failed to create output dir '{}'", parent.display())
                })?;
            }
            fs::write(output, &audio)
                .with_context(|| format!("failed to write output '{}'", output.display()))?;
            reporter.output_saved(output);
        }

        // 7. Play it, unless told not to.
        let mut played = false;
        if req.play {
            reporter.playing();
            self.player
                .play(&audio)
                .context("audio playback failed")?;
            played = true;
        }

        let outcome = TtsOutcome {
            cache_hit,
            cache_path,
            char_count,
            chunk_count: chunks.len(),
            audio_len: audio.len(),
            played,
            output_path: req.output_path,
        };
        reporter.finished(&outcome);
        Ok(outcome)
    }
}
