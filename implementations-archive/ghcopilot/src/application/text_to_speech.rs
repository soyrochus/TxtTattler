use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};

use crate::{
    config::ResolvedConfig,
    domain::{
        entities::{SynthesisOutcome, TtsRequest},
        ports::{AudioPlayer, ProgressHandle, Reporter, TextProcessor, TtsProvider},
    },
    infrastructure::file_readers::FileReaderRegistry,
};

pub const MAX_CHARS_PER_REQUEST: usize = 3_800;
pub const CACHE_PIPELINE_VERSION: &str = "txttattler-cache-v1";

pub struct TextToSpeechService {
    registry: Arc<FileReaderRegistry>,
    tts_provider: Arc<dyn TtsProvider>,
    audio_player: Arc<dyn AudioPlayer>,
    processors: Vec<Arc<dyn TextProcessor>>,
    reporter: Arc<dyn Reporter>,
}

impl TextToSpeechService {
    pub fn new(
        registry: Arc<FileReaderRegistry>,
        tts_provider: Arc<dyn TtsProvider>,
        audio_player: Arc<dyn AudioPlayer>,
        processors: Vec<Arc<dyn TextProcessor>>,
        reporter: Arc<dyn Reporter>,
    ) -> Self {
        Self {
            registry,
            tts_provider,
            audio_player,
            processors,
            reporter,
        }
    }

    pub async fn run(&self, config: &ResolvedConfig) -> Result<SynthesisOutcome> {
        self.reporter
            .status(&format!("Reading {} ...", config.input_path.display()));
        let raw_text = self.registry.read(&config.input_path)?;

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

        let cache_key = build_cache_key(
            &raw_text,
            config.voice.as_str(),
            config.model.as_str(),
            config.speed,
            &pipeline_version(&self.processors),
        );
        let cache_path = config.cache_dir.join(format!("{cache_key}.mp3"));

        self.reporter.detail(&format!(
            "Characters: raw={} cleaned={}",
            raw_text.chars().count(),
            processed.chars().count()
        ));

        let mut from_cache = false;
        let audio = if config.cache_enabled && !config.refresh && cache_path.exists() {
            self.reporter
                .status(&format!("Cache hit. Reusing {}", cache_path.display()));
            from_cache = true;
            fs::read(&cache_path)
                .with_context(|| format!("Failed to read cached MP3 at {}", cache_path.display()))?
        } else {
            let chunks = chunk_text(&processed, MAX_CHARS_PER_REQUEST);
            let chunk_progress = self
                .reporter
                .progress("Generating speech", chunks.len() as u64);
            let generated = self
                .generate_chunks(config, &chunks, chunk_progress)
                .await?;
            let audio = concat_mp3_segments(&generated);

            if config.cache_enabled {
                fs::create_dir_all(&config.cache_dir).with_context(|| {
                    format!(
                        "Failed to create cache directory {}",
                        config.cache_dir.display()
                    )
                })?;
                fs::write(&cache_path, &audio).with_context(|| {
                    format!("Failed to write cached MP3 to {}", cache_path.display())
                })?;
                self.reporter
                    .detail(&format!("Cached MP3 at {}", cache_path.display()));
            }

            audio
        };

        if let Some(output) = &config.output_path {
            write_output(output, &audio, from_cache.then_some(&cache_path))?;
            self.reporter
                .status(&format!("Saved MP3 to {}", output.display()));
        }

        if config.play_audio {
            self.reporter
                .status("Playback started. The tattler has opinions.");
            self.audio_player.play_mp3(audio.clone())?;
        }

        let outcome = SynthesisOutcome {
            cache_path: config.cache_enabled.then_some(cache_path),
            from_cache,
            output_path: config.output_path.clone(),
            chunks: chunk_text(&processed, MAX_CHARS_PER_REQUEST).len(),
            character_count: processed.chars().count(),
        };

        self.reporter.success(&format!(
            "Done. Voice={}, model={}, speed={:.2}x.",
            config.voice, config.model, config.speed
        ));

        Ok(outcome)
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
            };
            output.push(self.tts_provider.synthesize(&request).await?);
            progress.inc(1);
        }

        progress.finish_with_message("Speech generation finished");
        Ok(output)
    }
}

fn pipeline_version(processors: &[Arc<dyn TextProcessor>]) -> String {
    let processor_names = processors
        .iter()
        .map(|processor| processor.name())
        .collect::<Vec<_>>()
        .join(",");
    format!("{CACHE_PIPELINE_VERSION}:{processor_names}")
}

fn write_output(path: &Path, audio: &[u8], cache_path: Option<&PathBuf>) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory {}", parent.display()))?;
    }

    if let Some(cache_path) = cache_path
        && cache_path != path
    {
        fs::copy(cache_path, path).with_context(|| {
            format!(
                "Failed to copy cached MP3 from {} to {}",
                cache_path.display(),
                path.display()
            )
        })?;
        return Ok(());
    }

    fs::write(path, audio).with_context(|| format!("Failed to write MP3 to {}", path.display()))?;
    Ok(())
}

pub fn build_cache_key(
    raw_text: &str,
    voice: &str,
    model: &str,
    speed: f32,
    version: &str,
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
    format!("{:x}", hasher.finalize())
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

    if parts.is_empty() {
        bail_chunk(sentence, max_chars)
    } else {
        parts
    }
}

fn bail_chunk(sentence: &str, max_chars: usize) -> Vec<String> {
    sentence
        .chars()
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
        build_cache_key, chunk_text,
    };
    use crate::domain::ports::TextProcessor;

    #[test]
    fn cache_key_changes_when_voice_changes() {
        let first = build_cache_key("hello", "alloy", "tts-1", 1.0, "v1");
        let second = build_cache_key("hello", "nova", "tts-1", 1.0, "v1");
        assert_ne!(first, second);
    }

    #[test]
    fn chunker_keeps_each_chunk_under_limit() {
        let input = "Hello world. ".repeat(500);
        let chunks = chunk_text(&input, 200);
        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|chunk| chunk.chars().count() <= 200));
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
}
