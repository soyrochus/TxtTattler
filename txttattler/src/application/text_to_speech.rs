use std::{fs, path::Path, sync::Arc};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};

use crate::{
    config::ResolvedConfig,
    domain::{
        entities::{SpeechModelName, SynthesisOutcome, TtsRequest},
        ports::{
            AudioPlayer, DocumentReader, ProgressHandle, Reporter, TextProcessor, TtsProvider,
        },
    },
    utils::atomic_write,
};

pub const MAX_CHARS_PER_REQUEST: usize = 3_800;
pub const CACHE_PIPELINE_VERSION: &str = "txttattler-cache-v1";

pub struct TextToSpeechService {
    reader: Arc<dyn DocumentReader>,
    tts_provider: Arc<dyn TtsProvider>,
    audio_player: Arc<dyn AudioPlayer>,
    processors: Vec<Arc<dyn TextProcessor>>,
    reporter: Arc<dyn Reporter>,
}

impl TextToSpeechService {
    pub fn new(
        reader: Arc<dyn DocumentReader>,
        tts_provider: Arc<dyn TtsProvider>,
        audio_player: Arc<dyn AudioPlayer>,
        processors: Vec<Arc<dyn TextProcessor>>,
        reporter: Arc<dyn Reporter>,
    ) -> Self {
        Self {
            reader,
            tts_provider,
            audio_player,
            processors,
            reporter,
        }
    }

    pub async fn run(&self, config: &ResolvedConfig) -> Result<SynthesisOutcome> {
        self.reporter
            .status(&format!("Reading {} ...", config.input_path.display()));
        let raw_text = self.reader.read(&config.input_path)?;

        self.reporter.status("Cleaning text ...");
        let mut processed = raw_text.clone();
        for processor in &self.processors {
            self.reporter
                .detail(&format!("Applying text middleware: {}", processor.name()));
            processed = processor.process(processed)?;
        }

        if processed.trim().is_empty() {
            bail!(
                "There is no readable text left after cleanup. The tattler refuses to gossip into the void."
            );
        }

        let pipeline_version = pipeline_version(&self.processors);
        let cache_key = build_cache_key(
            &processed,
            config.voice.as_str(),
            config.model.as_str(),
            config.speed,
            &pipeline_version,
            config.instructions.as_deref(),
        );
        let cache_path = config.cache_dir.join(format!("{cache_key}.mp3"));
        let chunks = chunk_text(&processed, MAX_CHARS_PER_REQUEST);

        self.reporter.detail(&format!(
            "Characters: raw={} cleaned={}",
            raw_text.chars().count(),
            processed.chars().count()
        ));
        self.emit_compatibility_warnings(config);

        let mut from_cache = false;
        self.reporter.status("Checking cache ...");
        let audio = if config.cache_enabled && !config.refresh && cache_path.exists() {
            self.reporter
                .status(&format!("Cache hit. Reusing {}", cache_path.display()));
            from_cache = true;
            fs::read(&cache_path)
                .with_context(|| format!("Failed to read cached MP3 at {}", cache_path.display()))?
        } else {
            if config.refresh && cache_path.exists() {
                self.reporter
                    .status("Refresh requested. Regenerating cached speech ...");
            } else if config.cache_enabled {
                self.reporter.status("Cache miss. Generating speech ...");
            } else {
                self.reporter
                    .status("Cache disabled. Generating one-off speech ...");
            }
            let chunk_progress = self
                .reporter
                .progress("Generating speech", chunks.len() as u64);
            let generated = self
                .generate_chunks(config, &chunks, chunk_progress)
                .await?;
            let audio = concat_mp3_segments(&generated);

            if config.cache_enabled {
                self.reporter.status("Writing cache entry ...");
                atomic_write(&cache_path, &audio).with_context(|| {
                    format!("Failed to write cached MP3 to {}", cache_path.display())
                })?;
                self.reporter
                    .detail(&format!("Cached MP3 at {}", cache_path.display()));
            }

            audio
        };

        if let Some(output) = &config.output_path {
            self.reporter.status(if from_cache {
                "Copying cached MP3 to output ..."
            } else {
                "Writing MP3 output ..."
            });
            write_output(output, &audio, from_cache.then_some(cache_path.as_path()))?;
            self.reporter
                .status(&format!("Saved MP3 to {}", output.display()));
        }

        if config.play_audio {
            self.audio_player
                .play_mp3(audio.clone(), self.reporter.as_ref())
                .context("Failed to play generated audio.")?;
        }

        let outcome = SynthesisOutcome {
            input_path: config.input_path.clone(),
            cache_path: config.cache_enabled.then_some(cache_path),
            from_cache,
            output_path: config.output_path.clone(),
            chunks: chunks.len(),
            character_count: processed.chars().count(),
            voice: config.voice,
            model: config.model,
            speed: config.speed,
            instructions: config.instructions.clone(),
            playback: config.play_audio,
        };

        self.reporter.success(&format!(
            "Done. Voice={}, model={}, speed={:.2}x.",
            config.voice, config.model, config.speed
        ));
        if let Some(instructions) = &config.instructions {
            self.reporter.status(&format!(
                "Instructions: {}",
                format_instructions_for_summary(instructions, config.verbose)
            ));
        }

        Ok(outcome)
    }

    fn emit_compatibility_warnings(&self, config: &ResolvedConfig) {
        if config.instructions.is_some()
            && matches!(
                config.model,
                SpeechModelName::Tts1 | SpeechModelName::Tts1Hd
            )
        {
            self.reporter.warning(
                "--instructions is only supported by gpt-4o-mini-tts; tts-1 and tts-1-hd may ignore it.",
            );
        }

        if config.model == SpeechModelName::Gpt4oMiniTts
            && (config.speed - 1.0).abs() > f32::EPSILON
        {
            self.reporter.warning(
                "--speed is not reliably supported by gpt-4o-mini-tts; the requested speed will still be sent.",
            );
        }
    }

    async fn generate_chunks(
        &self,
        config: &ResolvedConfig,
        chunks: &[String],
        progress: Box<dyn ProgressHandle>,
    ) -> Result<Vec<Vec<u8>>> {
        let mut output = Vec::with_capacity(chunks.len());

        for (index, chunk) in chunks.iter().enumerate() {
            progress.set_message(&format!(
                "Chunk {}/{} ({} chars)",
                index + 1,
                chunks.len(),
                chunk.chars().count()
            ));
            self.reporter.detail(&format!(
                "Requesting chunk {}/{} with voice={}, model={}, speed={:.2}",
                index + 1,
                chunks.len(),
                config.voice,
                config.model,
                config.speed
            ));
            let request = TtsRequest {
                text: chunk.clone(),
                voice: config.voice,
                model: config.model,
                speed: config.speed,
                instructions: config.instructions.clone(),
            };
            output.push(self.tts_provider.synthesize(&request).await?);
            progress.inc(1);
        }

        progress.finish_with_message("Speech generation finished");
        Ok(output)
    }
}

pub fn pipeline_version(processors: &[Arc<dyn TextProcessor>]) -> String {
    let processor_names = processors
        .iter()
        .map(|processor| processor.name())
        .collect::<Vec<_>>()
        .join(",");
    format!("{CACHE_PIPELINE_VERSION}:{processor_names}")
}

fn write_output(path: &Path, audio: &[u8], cache_path: Option<&Path>) -> Result<()> {
    if let Some(cache_path) = cache_path
        && cache_path != path
    {
        let cached_audio = fs::read(cache_path).with_context(|| {
            format!(
                "Failed to read cached MP3 from {} while writing output",
                cache_path.display()
            )
        })?;
        atomic_write(path, &cached_audio).with_context(|| {
            format!(
                "Failed to copy cached MP3 from {} to {}",
                cache_path.display(),
                path.display()
            )
        })?;
        return Ok(());
    }

    atomic_write(path, audio)
        .with_context(|| format!("Failed to write MP3 to {}", path.display()))?;
    Ok(())
}

pub fn build_cache_key(
    raw_text: &str,
    voice: &str,
    model: &str,
    speed: f32,
    version: &str,
    instructions: Option<&str>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_text.as_bytes());
    hasher.update([0]);
    hasher.update(voice.as_bytes());
    hasher.update([0]);
    hasher.update(model.as_bytes());
    hasher.update([0]);
    hasher.update(format!("{speed:.3}").as_bytes());
    hasher.update([0]);
    hasher.update(version.as_bytes());
    hasher.update([0]);
    if let Some(instructions) = instructions {
        hasher.update([1]);
        hasher.update(instructions.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

pub fn format_instructions_for_summary(instructions: &str, verbose: bool) -> String {
    if verbose {
        return instructions.to_string();
    }

    const MAX_DISPLAY_CHARS: usize = 80;
    if instructions.chars().count() <= MAX_DISPLAY_CHARS {
        return instructions.to_string();
    }

    let truncated = instructions
        .chars()
        .take(MAX_DISPLAY_CHARS - 1)
        .collect::<String>();
    format!("{truncated}…")
}

pub fn concat_mp3_segments(segments: &[Vec<u8>]) -> Vec<u8> {
    let total_len = segments.iter().map(Vec::len).sum();
    let mut output = Vec::with_capacity(total_len);
    for segment in segments {
        output.extend_from_slice(segment);
    }
    output
}

pub fn chunk_text(text: &str, max_chars: usize) -> Vec<String> {
    if text.trim().is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let paragraphs = text
        .split("\n\n")
        .map(str::trim)
        .filter(|paragraph| !paragraph.is_empty());

    for paragraph in paragraphs {
        if paragraph.chars().count() <= max_chars {
            push_or_append(&mut chunks, paragraph, max_chars);
            continue;
        }

        for sentence in split_long_paragraph(paragraph) {
            if sentence.chars().count() <= max_chars {
                push_or_append(&mut chunks, &sentence, max_chars);
            } else {
                for word_chunk in split_by_words(&sentence, max_chars) {
                    push_or_append(&mut chunks, &word_chunk, max_chars);
                }
            }
        }
    }

    chunks
}

fn push_or_append(chunks: &mut Vec<String>, content: &str, max_chars: usize) {
    if let Some(last) = chunks.last_mut() {
        let proposed = format!("{last}\n\n{content}");
        if proposed.chars().count() <= max_chars {
            *last = proposed;
            return;
        }
    }

    chunks.push(content.to_string());
}

fn split_long_paragraph(paragraph: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut buffer = String::new();

    for ch in paragraph.chars() {
        buffer.push(ch);
        if matches!(ch, '.' | '!' | '?' | ';') {
            if !buffer.trim().is_empty() {
                parts.push(buffer.trim().to_string());
            }
            buffer.clear();
        }
    }

    if !buffer.trim().is_empty() {
        parts.push(buffer.trim().to_string());
    }

    if parts.is_empty() {
        vec![paragraph.trim().to_string()]
    } else {
        parts
    }
}

fn split_by_words(sentence: &str, max_chars: usize) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();

    for word in sentence.split_whitespace() {
        if word.chars().count() > max_chars {
            if !current.is_empty() {
                parts.push(std::mem::take(&mut current));
            }
            parts.extend(char_chunks(word, max_chars));
            continue;
        }

        if current.is_empty() {
            current.push_str(word);
            continue;
        }

        let proposed = format!("{current} {word}");
        if proposed.chars().count() <= max_chars {
            current = proposed;
        } else {
            parts.push(current);
            current = word.to_string();
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

fn char_chunks(text: &str, max_chars: usize) -> Vec<String> {
    text.chars()
        .collect::<Vec<_>>()
        .chunks(max_chars)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect()
}

#[derive(Debug, Default)]
pub struct CollapseWhitespaceProcessor;

impl TextProcessor for CollapseWhitespaceProcessor {
    fn name(&self) -> &'static str {
        "collapse-whitespace"
    }

    fn process(&self, input: String) -> Result<String> {
        Ok(input
            .lines()
            .map(str::trim)
            .collect::<Vec<_>>()
            .join("\n")
            .replace("\r\n", "\n"))
    }
}

#[derive(Debug, Default)]
pub struct ReduceBlankLinesProcessor;

impl TextProcessor for ReduceBlankLinesProcessor {
    fn name(&self) -> &'static str {
        "reduce-blank-lines"
    }

    fn process(&self, input: String) -> Result<String> {
        let mut result = String::new();
        let mut blank_run = 0usize;

        for line in input.lines() {
            if line.trim().is_empty() {
                blank_run += 1;
                if blank_run <= 1 {
                    result.push('\n');
                }
            } else {
                blank_run = 0;
                if !result.is_empty() && !result.ends_with('\n') {
                    result.push('\n');
                }
                result.push_str(line.trim_end());
            }
        }

        Ok(result.trim().to_string())
    }
}

#[derive(Debug, Default)]
pub struct StripMarkdownProcessor;

impl TextProcessor for StripMarkdownProcessor {
    fn name(&self) -> &'static str {
        "strip-markdown-lite"
    }

    fn process(&self, input: String) -> Result<String> {
        let cleaned = input
            .replace("```", "")
            .replace(['#', '*', '`', '_', '>'], "")
            .replace('[', "")
            .replace(']', "")
            .replace("(", "")
            .replace(")", "");
        Ok(cleaned)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CollapseWhitespaceProcessor, ReduceBlankLinesProcessor, StripMarkdownProcessor,
        TextToSpeechService, build_cache_key, chunk_text, format_instructions_for_summary,
        pipeline_version,
    };
    use crate::{
        config::{OpenAiRuntimeConfig, ProviderConfig, ResolvedConfig},
        domain::{
            entities::{SpeechModelName, TtsRequest, VoiceName},
            ports::{
                AudioPlayer, DocumentReader, ProgressHandle, Reporter, TextProcessor, TtsProvider,
            },
        },
        utils::SilentReporter,
    };
    use anyhow::Result;
    use async_trait::async_trait;
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::{
            Arc, Mutex,
            atomic::{AtomicUsize, Ordering},
        },
    };
    use tempfile::TempDir;

    #[derive(Clone)]
    struct StaticReader {
        text: String,
    }

    impl DocumentReader for StaticReader {
        fn read(&self, _path: &Path) -> Result<String> {
            Ok(self.text.clone())
        }
    }

    struct CountingTtsProvider {
        calls: Arc<AtomicUsize>,
        requests: Option<Arc<Mutex<Vec<TtsRequest>>>>,
    }

    #[async_trait]
    impl TtsProvider for CountingTtsProvider {
        async fn synthesize(&self, request: &TtsRequest) -> Result<Vec<u8>> {
            if let Some(requests) = &self.requests {
                requests.lock().unwrap().push(request.clone());
            }
            let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
            Ok(format!("audio-{call}").into_bytes())
        }
    }

    #[derive(Default)]
    struct RecordingAudioPlayer {
        plays: Mutex<usize>,
    }

    impl AudioPlayer for RecordingAudioPlayer {
        fn play_mp3(&self, _audio: Vec<u8>, _reporter: &dyn Reporter) -> Result<()> {
            *self.plays.lock().unwrap() += 1;
            Ok(())
        }
    }

    #[derive(Default)]
    struct RecordingReporter {
        warnings: Mutex<Vec<String>>,
        statuses: Mutex<Vec<String>>,
    }

    impl Reporter for RecordingReporter {
        fn status(&self, message: &str) {
            self.statuses.lock().unwrap().push(message.to_string());
        }

        fn detail(&self, _message: &str) {}

        fn warning(&self, message: &str) {
            self.warnings.lock().unwrap().push(message.to_string());
        }

        fn success(&self, message: &str) {
            self.statuses.lock().unwrap().push(message.to_string());
        }

        fn progress(&self, _message: &str, _len: u64) -> Box<dyn ProgressHandle> {
            Box::new(NoopProgressHandle)
        }

        fn playback(
            &self,
            _total: Option<std::time::Duration>,
        ) -> Box<dyn crate::domain::ports::PlaybackProgressHandle> {
            Box::new(NoopPlaybackProgressHandle)
        }
    }

    struct NoopProgressHandle;

    impl ProgressHandle for NoopProgressHandle {
        fn inc(&self, _delta: u64) {}

        fn set_message(&self, _message: &str) {}

        fn finish_with_message(&self, _message: &str) {}
    }

    struct NoopPlaybackProgressHandle;

    impl crate::domain::ports::PlaybackProgressHandle for NoopPlaybackProgressHandle {
        fn tick(&self, _elapsed: std::time::Duration) {}

        fn finish(&self) {}
    }

    fn test_config(tempdir: &TempDir) -> ResolvedConfig {
        ResolvedConfig {
            input_path: tempdir.path().join("input.txt"),
            voice: VoiceName::Alloy,
            model: SpeechModelName::Tts1,
            speed: 1.0,
            instructions: None,
            output_path: None,
            play_audio: false,
            cache_enabled: true,
            refresh: false,
            cache_dir: tempdir.path().join("cache"),
            verbose: false,
            strip_markdown: false,
            provider: ProviderConfig::OpenAi(OpenAiRuntimeConfig {
                api_key: "test-key".to_string(),
                api_base: None,
                org_id: None,
                project_id: None,
            }),
        }
    }

    fn service(text: &str, calls: Arc<AtomicUsize>) -> TextToSpeechService {
        service_with_player(text, calls, Arc::new(RecordingAudioPlayer::default()))
    }

    fn service_with_player(
        text: &str,
        calls: Arc<AtomicUsize>,
        audio_player: Arc<RecordingAudioPlayer>,
    ) -> TextToSpeechService {
        TextToSpeechService::new(
            Arc::new(StaticReader {
                text: text.to_string(),
            }),
            Arc::new(CountingTtsProvider {
                calls,
                requests: None,
            }),
            audio_player,
            vec![
                Arc::new(CollapseWhitespaceProcessor),
                Arc::new(ReduceBlankLinesProcessor),
            ],
            Arc::new(SilentReporter),
        )
    }

    fn service_with_request_recorder(
        text: &str,
        calls: Arc<AtomicUsize>,
        requests: Arc<Mutex<Vec<TtsRequest>>>,
        reporter: Arc<dyn Reporter>,
    ) -> TextToSpeechService {
        TextToSpeechService::new(
            Arc::new(StaticReader {
                text: text.to_string(),
            }),
            Arc::new(CountingTtsProvider {
                calls,
                requests: Some(requests),
            }),
            Arc::new(RecordingAudioPlayer::default()),
            vec![
                Arc::new(CollapseWhitespaceProcessor),
                Arc::new(ReduceBlankLinesProcessor),
            ],
            reporter,
        )
    }

    #[test]
    fn cache_key_changes_when_voice_changes() {
        let first = build_cache_key("hello", "alloy", "tts-1", 1.0, "v1", None);
        let second = build_cache_key("hello", "nova", "tts-1", 1.0, "v1", None);
        assert_ne!(first, second);
    }

    #[test]
    fn cache_key_changes_for_output_affecting_fields() {
        let base = build_cache_key("hello", "alloy", "tts-1", 1.0, "v1", None);
        assert_ne!(
            base,
            build_cache_key("goodbye", "alloy", "tts-1", 1.0, "v1", None)
        );
        assert_ne!(
            base,
            build_cache_key("hello", "alloy", "tts-1-hd", 1.0, "v1", None)
        );
        assert_ne!(
            base,
            build_cache_key("hello", "alloy", "tts-1", 1.1, "v1", None)
        );
        assert_ne!(
            base,
            build_cache_key("hello", "alloy", "tts-1", 1.0, "v2", None)
        );
    }

    #[test]
    fn cache_key_uses_null_separators() {
        let separated = build_cache_key("ab", "c", "tts-1", 1.0, "v1", None);
        let ambiguous = build_cache_key("a", "bc", "tts-1", 1.0, "v1", None);
        assert_ne!(separated, ambiguous);
    }

    #[test]
    fn cache_key_includes_instructions() {
        let dutch = build_cache_key("hello", "alloy", "tts-1", 1.0, "v1", Some("Speak Dutch."));
        let english = build_cache_key("hello", "alloy", "tts-1", 1.0, "v1", Some("Speak English."));
        assert_ne!(dutch, english);
    }

    #[test]
    fn cache_key_distinguishes_none_and_empty_instructions() {
        let none = build_cache_key("hello", "alloy", "tts-1", 1.0, "v1", None);
        let empty = build_cache_key("hello", "alloy", "tts-1", 1.0, "v1", Some(""));
        assert_ne!(none, empty);
    }

    #[test]
    fn identical_cache_key_inputs_with_instructions_are_stable() {
        let first = build_cache_key("hello", "alloy", "tts-1", 1.0, "v1", Some("Speak Dutch."));
        let second = build_cache_key("hello", "alloy", "tts-1", 1.0, "v1", Some("Speak Dutch."));
        assert_eq!(first, second);
    }

    #[test]
    fn cache_key_uses_full_instructions() {
        let long = "Speak in a warm, precise, clearly articulated Dutch style for the full passage with careful pacing and calm emphasis.";
        let truncated = format_instructions_for_summary(long, false);

        assert_ne!(
            build_cache_key("hello", "alloy", "tts-1", 1.0, "v1", Some(long)),
            build_cache_key("hello", "alloy", "tts-1", 1.0, "v1", Some(&truncated))
        );
    }

    #[test]
    fn instructions_summary_truncates_only_in_normal_mode() {
        let instructions = "a".repeat(81);
        assert_eq!(
            format_instructions_for_summary(&instructions, false)
                .chars()
                .count(),
            80
        );
        assert!(format_instructions_for_summary(&instructions, false).ends_with('…'));
        assert_eq!(
            format_instructions_for_summary(&instructions, true),
            instructions
        );
    }

    #[test]
    fn chunker_keeps_each_chunk_under_limit() {
        let input = "Hello world. ".repeat(500);
        let chunks = chunk_text(&input, 200);
        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|chunk| chunk.chars().count() <= 200));
    }

    #[test]
    fn chunker_returns_short_input_as_one_chunk() {
        assert_eq!(chunk_text("Short sentence.", 200), vec!["Short sentence."]);
    }

    #[test]
    fn chunker_preserves_sentence_punctuation() {
        let chunks = chunk_text("First sentence. Second sentence!", 16);
        assert!(chunks.iter().any(|chunk| chunk.ends_with('.')));
        assert!(chunks.iter().any(|chunk| chunk.ends_with('!')));
    }

    #[test]
    fn chunker_splits_long_words_by_character() {
        let chunks = chunk_text("supercalifragilistic", 5);
        assert!(chunks.iter().all(|chunk| chunk.chars().count() <= 5));
        assert_eq!(chunks.concat(), "supercalifragilistic");
    }

    #[test]
    fn whitespace_processor_trims_lines() {
        let processor = CollapseWhitespaceProcessor;
        let result = processor
            .process("  hello  \n   world   ".to_string())
            .unwrap();
        assert_eq!(result, "hello\nworld");
    }

    #[test]
    fn blank_line_processor_reduces_runs() {
        let processor = ReduceBlankLinesProcessor;
        let result = processor
            .process("hello\n\n\nworld\n\n\nagain".to_string())
            .unwrap();
        assert_eq!(result, "hello\nworld\nagain");
    }

    #[test]
    fn markdown_processor_strips_common_tokens() {
        let processor = StripMarkdownProcessor;
        let result = processor
            .process("# Hello **world** [link](https://example.com)".to_string())
            .unwrap();
        assert!(!result.contains('#'));
        assert!(!result.contains('*'));
        assert!(!result.contains('['));
    }

    #[test]
    fn processor_names_form_pipeline_version() {
        let processors: Vec<Arc<dyn TextProcessor>> = vec![
            Arc::new(CollapseWhitespaceProcessor),
            Arc::new(ReduceBlankLinesProcessor),
            Arc::new(StripMarkdownProcessor),
        ];

        let version = pipeline_version(&processors);

        assert!(version.contains("collapse-whitespace"));
        assert!(version.contains("reduce-blank-lines"));
        assert!(version.contains("strip-markdown-lite"));
    }

    #[tokio::test]
    async fn cache_reuse_avoids_second_tts_call() {
        let tempdir = TempDir::new().unwrap();
        let config = test_config(&tempdir);
        let calls = Arc::new(AtomicUsize::new(0));
        let service = service("hello world", calls.clone());

        let first = service.run(&config).await.unwrap();
        let second = service.run(&config).await.unwrap();

        assert!(!first.from_cache);
        assert!(second.from_cache);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn requests_include_resolved_instructions() {
        let tempdir = TempDir::new().unwrap();
        let mut config = test_config(&tempdir);
        config.instructions = Some("Speak in Dutch.".to_string());
        let calls = Arc::new(AtomicUsize::new(0));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let service = service_with_request_recorder(
            "hello world",
            calls,
            requests.clone(),
            Arc::new(SilentReporter),
        );

        service.run(&config).await.unwrap();

        assert_eq!(
            requests.lock().unwrap()[0].instructions,
            Some("Speak in Dutch.".to_string())
        );
    }

    #[tokio::test]
    async fn requests_include_none_when_instructions_absent() {
        let tempdir = TempDir::new().unwrap();
        let config = test_config(&tempdir);
        let calls = Arc::new(AtomicUsize::new(0));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let service = service_with_request_recorder(
            "hello world",
            calls,
            requests.clone(),
            Arc::new(SilentReporter),
        );

        service.run(&config).await.unwrap();

        assert_eq!(requests.lock().unwrap()[0].instructions, None);
    }

    #[tokio::test]
    async fn cache_key_uses_instructions_during_runs() {
        let tempdir = TempDir::new().unwrap();
        let mut config = test_config(&tempdir);
        config.instructions = Some("Speak in Dutch.".to_string());
        let calls = Arc::new(AtomicUsize::new(0));
        let service = service("hello world", calls.clone());

        let first = service.run(&config).await.unwrap();
        let second = service.run(&config).await.unwrap();
        config.instructions = Some("Speak in English.".to_string());
        let third = service.run(&config).await.unwrap();

        assert!(!first.from_cache);
        assert!(second.from_cache);
        assert!(!third.from_cache);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn instructions_with_classic_models_warns() {
        for model in [SpeechModelName::Tts1, SpeechModelName::Tts1Hd] {
            let tempdir = TempDir::new().unwrap();
            let mut config = test_config(&tempdir);
            config.model = model;
            config.instructions = Some("Speak in Dutch.".to_string());
            let calls = Arc::new(AtomicUsize::new(0));
            let requests = Arc::new(Mutex::new(Vec::new()));
            let reporter = Arc::new(RecordingReporter::default());
            let service =
                service_with_request_recorder("hello world", calls, requests, reporter.clone());

            service.run(&config).await.unwrap();

            assert_eq!(reporter.warnings.lock().unwrap().len(), 1);
            assert!(
                reporter.warnings.lock().unwrap()[0].contains("only supported by gpt-4o-mini-tts")
            );
        }
    }

    #[tokio::test]
    async fn instructions_with_gpt4o_mini_tts_does_not_warn() {
        let tempdir = TempDir::new().unwrap();
        let mut config = test_config(&tempdir);
        config.model = SpeechModelName::Gpt4oMiniTts;
        config.instructions = Some("Speak in Dutch.".to_string());
        let calls = Arc::new(AtomicUsize::new(0));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let reporter = Arc::new(RecordingReporter::default());
        let service =
            service_with_request_recorder("hello world", calls, requests, reporter.clone());

        service.run(&config).await.unwrap();

        assert!(reporter.warnings.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn non_default_speed_with_gpt4o_mini_tts_warns() {
        let tempdir = TempDir::new().unwrap();
        let mut config = test_config(&tempdir);
        config.model = SpeechModelName::Gpt4oMiniTts;
        config.speed = 1.5;
        let calls = Arc::new(AtomicUsize::new(0));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let reporter = Arc::new(RecordingReporter::default());
        let service =
            service_with_request_recorder("hello world", calls, requests, reporter.clone());

        service.run(&config).await.unwrap();

        assert_eq!(reporter.warnings.lock().unwrap().len(), 1);
        assert!(
            reporter.warnings.lock().unwrap()[0]
                .contains("not reliably supported by gpt-4o-mini-tts")
        );
    }

    #[tokio::test]
    async fn non_default_speed_with_tts1_does_not_warn() {
        let tempdir = TempDir::new().unwrap();
        let mut config = test_config(&tempdir);
        config.model = SpeechModelName::Tts1;
        config.speed = 1.5;
        let calls = Arc::new(AtomicUsize::new(0));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let reporter = Arc::new(RecordingReporter::default());
        let service =
            service_with_request_recorder("hello world", calls, requests, reporter.clone());

        service.run(&config).await.unwrap();

        assert!(reporter.warnings.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn instructions_appear_in_summary_when_set() {
        let tempdir = TempDir::new().unwrap();
        let mut config = test_config(&tempdir);
        config.model = SpeechModelName::Gpt4oMiniTts;
        config.instructions = Some("Speak in Dutch.".to_string());
        let calls = Arc::new(AtomicUsize::new(0));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let reporter = Arc::new(RecordingReporter::default());
        let service =
            service_with_request_recorder("hello world", calls, requests, reporter.clone());

        service.run(&config).await.unwrap();

        assert!(
            reporter
                .statuses
                .lock()
                .unwrap()
                .iter()
                .any(|message| message == "Instructions: Speak in Dutch.")
        );
    }

    #[tokio::test]
    async fn instructions_absent_from_summary_when_unset() {
        let tempdir = TempDir::new().unwrap();
        let config = test_config(&tempdir);
        let calls = Arc::new(AtomicUsize::new(0));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let reporter = Arc::new(RecordingReporter::default());
        let service =
            service_with_request_recorder("hello world", calls, requests, reporter.clone());

        service.run(&config).await.unwrap();

        assert!(
            reporter
                .statuses
                .lock()
                .unwrap()
                .iter()
                .all(|message| !message.starts_with("Instructions:"))
        );
    }

    #[tokio::test]
    async fn refresh_ignores_existing_cache() {
        let tempdir = TempDir::new().unwrap();
        let mut config = test_config(&tempdir);
        let calls = Arc::new(AtomicUsize::new(0));
        let service = service("hello world", calls.clone());

        service.run(&config).await.unwrap();
        config.refresh = true;
        let refreshed = service.run(&config).await.unwrap();

        assert!(!refreshed.from_cache);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn no_cache_skips_reads_and_writes() {
        let tempdir = TempDir::new().unwrap();
        let mut config = test_config(&tempdir);
        config.cache_enabled = false;
        let calls = Arc::new(AtomicUsize::new(0));
        let service = service("hello world", calls.clone());

        let outcome = service.run(&config).await.unwrap();

        assert!(!outcome.from_cache);
        assert!(outcome.cache_path.is_none());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(!config.cache_dir.exists());
    }

    #[tokio::test]
    async fn cache_hit_writes_output_without_tts() {
        let tempdir = TempDir::new().unwrap();
        let mut config = test_config(&tempdir);
        let output = tempdir.path().join("out.mp3");
        let calls = Arc::new(AtomicUsize::new(0));
        let service = service("hello world", calls.clone());

        service.run(&config).await.unwrap();
        config.output_path = Some(output.clone());
        let outcome = service.run(&config).await.unwrap();

        assert!(outcome.from_cache);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(fs::read(output).unwrap(), b"audio-1");
    }

    #[tokio::test]
    async fn silent_reporter_allows_use_case_without_stdout_capture() {
        let tempdir = TempDir::new().unwrap();
        let config = test_config(&tempdir);
        let calls = Arc::new(AtomicUsize::new(0));
        let service = service("hello world", calls);

        let outcome = service.run(&config).await.unwrap();

        assert_eq!(
            outcome.input_path,
            PathBuf::from(tempdir.path().join("input.txt"))
        );
        assert_eq!(outcome.voice, VoiceName::Alloy);
        assert_eq!(outcome.model, SpeechModelName::Tts1);
        assert_eq!(outcome.speed, 1.0);
        assert!(!outcome.playback);
        assert_eq!(outcome.chunks, 1);
        assert_eq!(outcome.character_count, "hello world".chars().count());
    }

    #[tokio::test]
    async fn no_play_suppresses_audio_player() {
        let tempdir = TempDir::new().unwrap();
        let config = test_config(&tempdir);
        let calls = Arc::new(AtomicUsize::new(0));
        let audio_player = Arc::new(RecordingAudioPlayer::default());
        let service = service_with_player("hello world", calls, audio_player.clone());

        let outcome = service.run(&config).await.unwrap();

        assert!(!outcome.playback);
        assert_eq!(*audio_player.plays.lock().unwrap(), 0);
    }
}
