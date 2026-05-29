//! End-to-end exercise of the `TextToSpeech` use-case with fake adapters, so we
//! verify the orchestration (caching, chunking, output, playback gating) without
//! ever calling OpenAI or touching real speakers.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;

use txttattler::application::text_to_speech::{
    ProgressReporter, SilentReporter, TextToSpeech, TtsRequest,
};
use txttattler::domain::entities::TtsOptions;
use txttattler::domain::ports::{AudioPlayer, FileReader, TextProcessor, TtsProvider};

/// A reader that returns canned text and counts how often it was hit.
struct FakeReader {
    text: String,
    reads: Arc<AtomicUsize>,
}

impl FileReader for FakeReader {
    fn read(&self, _path: &Path) -> anyhow::Result<String> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        Ok(self.text.clone())
    }
}

/// Passes text through unchanged.
struct PassthroughProcessor;
impl TextProcessor for PassthroughProcessor {
    fn process(&self, text: &str) -> String {
        text.to_string()
    }
}

/// A TTS provider that fabricates deterministic "audio" and counts API calls.
struct CountingTts {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl TtsProvider for CountingTts {
    async fn synthesize(&self, text: &str, _options: &TtsOptions) -> anyhow::Result<Vec<u8>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(format!("AUDIO[{text}]").into_bytes())
    }
}

/// A player that records how many times it was asked to play.
struct CountingPlayer {
    plays: Arc<AtomicUsize>,
}

impl AudioPlayer for CountingPlayer {
    fn play(&self, _audio: &[u8]) -> anyhow::Result<()> {
        self.plays.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

struct Harness {
    use_case: TextToSpeech,
    reads: Arc<AtomicUsize>,
    tts_calls: Arc<AtomicUsize>,
    plays: Arc<AtomicUsize>,
}

fn harness(text: &str) -> Harness {
    let reads = Arc::new(AtomicUsize::new(0));
    let tts_calls = Arc::new(AtomicUsize::new(0));
    let plays = Arc::new(AtomicUsize::new(0));

    let use_case = TextToSpeech::new(
        Arc::new(FakeReader {
            text: text.to_string(),
            reads: reads.clone(),
        }),
        Arc::new(PassthroughProcessor),
        Arc::new(CountingTts {
            calls: tts_calls.clone(),
        }),
        Arc::new(CountingPlayer {
            plays: plays.clone(),
        }),
    );

    Harness {
        use_case,
        reads,
        tts_calls,
        plays,
    }
}

fn request(cache_dir: &Path, play: bool, use_cache: bool, refresh: bool) -> TtsRequest {
    TtsRequest {
        input_path: cache_dir.join("input.txt"),
        output_path: None,
        cache_dir: cache_dir.to_path_buf(),
        options: TtsOptions::default(),
        play,
        use_cache,
        refresh,
    }
}

#[tokio::test]
async fn synthesizes_caches_and_plays() {
    let dir = tempfile::tempdir().unwrap();
    let h = harness("Hello, world. This is the tattler.");
    let reporter: &dyn ProgressReporter = &SilentReporter;

    let outcome = h
        .use_case
        .run(request(dir.path(), true, true, false), reporter)
        .await
        .unwrap();

    assert!(!outcome.cache_hit);
    assert!(outcome.played);
    assert!(outcome.cache_path.exists(), "cache file should be written");
    assert_eq!(h.reads.load(Ordering::SeqCst), 1);
    assert_eq!(h.tts_calls.load(Ordering::SeqCst), 1);
    assert_eq!(h.plays.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn second_run_hits_the_cache_and_skips_the_api() {
    let dir = tempfile::tempdir().unwrap();
    let h = harness("Same content, same options, same key.");
    let reporter: &dyn ProgressReporter = &SilentReporter;

    let first = h
        .use_case
        .run(request(dir.path(), false, true, false), reporter)
        .await
        .unwrap();
    assert!(!first.cache_hit);
    assert_eq!(h.tts_calls.load(Ordering::SeqCst), 1);

    let second = h
        .use_case
        .run(request(dir.path(), false, true, false), reporter)
        .await
        .unwrap();
    assert!(second.cache_hit, "identical run should hit the cache");
    assert_eq!(
        h.tts_calls.load(Ordering::SeqCst),
        1,
        "no extra API calls on a cache hit"
    );
}

#[tokio::test]
async fn refresh_forces_regeneration() {
    let dir = tempfile::tempdir().unwrap();
    let h = harness("Refresh me, please.");
    let reporter: &dyn ProgressReporter = &SilentReporter;

    h.use_case
        .run(request(dir.path(), false, true, false), reporter)
        .await
        .unwrap();
    let refreshed = h
        .use_case
        .run(request(dir.path(), false, true, true), reporter)
        .await
        .unwrap();

    assert!(!refreshed.cache_hit);
    assert_eq!(h.tts_calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn no_cache_disables_reads_and_writes() {
    let dir = tempfile::tempdir().unwrap();
    let h = harness("Ephemeral gossip.");
    let reporter: &dyn ProgressReporter = &SilentReporter;

    let outcome = h
        .use_case
        .run(request(dir.path(), false, false, false), reporter)
        .await
        .unwrap();

    assert!(!outcome.cache_hit);
    assert!(
        !outcome.cache_path.exists(),
        "no cache file should be written with --no-cache"
    );
}

#[tokio::test]
async fn no_play_still_saves_output() {
    let dir = tempfile::tempdir().unwrap();
    let h = harness("Save but stay quiet.");
    let reporter: &dyn ProgressReporter = &SilentReporter;

    let output = dir.path().join("out.mp3");
    let mut req = request(dir.path(), false, true, false);
    req.output_path = Some(output.clone());

    let outcome = h.use_case.run(req, reporter).await.unwrap();

    assert!(!outcome.played);
    assert_eq!(h.plays.load(Ordering::SeqCst), 0);
    assert!(output.exists(), "output MP3 should be saved even with --no-play");
    assert_eq!(outcome.output_path, Some(output));
}

#[tokio::test]
async fn long_text_is_split_into_multiple_api_calls() {
    let dir = tempfile::tempdir().unwrap();
    // A long sentence-rich blob that will exceed the chunk limit many times over.
    let blob = "This is a sentence. ".repeat(1000);
    let h = harness(&blob);
    let reporter: &dyn ProgressReporter = &SilentReporter;

    let outcome = h
        .use_case
        .run(request(dir.path(), false, true, false), reporter)
        .await
        .unwrap();

    assert!(outcome.chunk_count > 1, "long text should produce many chunks");
    assert_eq!(
        h.tts_calls.load(Ordering::SeqCst),
        outcome.chunk_count,
        "one API call per chunk"
    );
}
