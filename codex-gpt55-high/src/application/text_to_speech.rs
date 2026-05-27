use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use tracing::{debug, info};

use crate::domain::entities::{MAX_TTS_CHARS, SpeechReport, TEXT_PROCESSING_VERSION, TtsOptions};
use crate::domain::ports::{AudioPlayer, TextProcessor, TtsProvider};
use crate::infrastructure::file_readers::FileReaderRegistry;
use crate::utils::{atomic_write, cache_key, chunk_text, concat_mp3_segments};

#[derive(Clone)]
pub struct TextToSpeechUseCase {
    readers: Arc<FileReaderRegistry>,
    tts: Arc<dyn TtsProvider>,
    player: Arc<dyn AudioPlayer>,
    processor: Arc<dyn TextProcessor>,
}

#[derive(Debug, Clone)]
pub struct SpeechCommand {
    pub input: PathBuf,
    pub options: TtsOptions,
    pub output: Option<PathBuf>,
    pub no_play: bool,
    pub no_cache: bool,
    pub refresh: bool,
    pub cache_dir: PathBuf,
    pub verbose: bool,
}

impl TextToSpeechUseCase {
    pub fn new(
        readers: Arc<FileReaderRegistry>,
        tts: Arc<dyn TtsProvider>,
        player: Arc<dyn AudioPlayer>,
        processor: Arc<dyn TextProcessor>,
    ) -> Self {
        Self {
            readers,
            tts,
            player,
            processor,
        }
    }

    pub async fn execute(&self, command: SpeechCommand) -> Result<SpeechReport> {
        let started = std::time::Instant::now();
        println!(
            "{} {}",
            style("Reading").cyan().bold(),
            command.input.display()
        );
        let text = self.read_file(&command.input)?;
        println!(
            "{} {} chars",
            style("Cleaning").cyan().bold(),
            text.chars().count()
        );
        let processed = self.processor.process(&text)?;
        if processed.cleaned.trim().is_empty() {
            bail!("the input file has no speakable text after cleanup");
        }

        let chunks = chunk_text(&processed.cleaned, MAX_TTS_CHARS);
        println!(
            "{} voice={}, model={}, speed={}, chunks={}",
            style("TTS").cyan().bold(),
            command.options.voice,
            command.options.model,
            command.options.speed,
            chunks.len()
        );

        let key = cache_key(
            &processed.cleaned,
            &command.options,
            TEXT_PROCESSING_VERSION,
        );
        let cache_path = command.cache_dir.join(format!("{key}.mp3"));

        let (mp3, cache_hit) = self
            .resolve_audio(&command, &chunks, &cache_path)
            .await
            .with_context(|| {
                format!(
                    "failed to generate speech for {} chunk(s); the tattler got tongue-tied",
                    chunks.len()
                )
            })?;

        if let Some(output) = &command.output {
            copy_to_output(&mp3, output)
                .with_context(|| format!("failed to save output MP3 to {}", output.display()))?;
            println!("{} {}", style("Saved").green().bold(), output.display());
        }

        if !command.no_play {
            println!("{}", style("Playing").cyan().bold());
            self.player.play(&mp3)?;
        }

        let elapsed = started.elapsed();
        info!(?elapsed, cache_hit, "speech command completed");
        if command.verbose {
            println!("{} {:?}", style("Elapsed").dim(), elapsed);
        }

        Ok(SpeechReport {
            character_count: processed.cleaned.chars().count(),
            chunk_count: chunks.len(),
            output_path: command.output,
            cache_path: (!command.no_cache).then_some(cache_path),
            cache_hit,
        })
    }

    fn read_file(&self, input: &Path) -> Result<String> {
        let extension = input
            .extension()
            .and_then(|extension| extension.to_str())
            .context("input file must have an extension; v1 only knows .txt")?;
        let reader = self.readers.reader_for(extension).with_context(|| {
            format!("unsupported file extension .{extension}; v1 supports .txt")
        })?;
        reader.read(input)
    }

    async fn resolve_audio(
        &self,
        command: &SpeechCommand,
        chunks: &[String],
        cache_path: &Path,
    ) -> Result<(Vec<u8>, bool)> {
        if !command.no_cache && !command.refresh && cache_path.exists() {
            println!(
                "{} {}",
                style("Cache hit").green().bold(),
                cache_path.display()
            );
            return Ok((fs::read(cache_path)?, true));
        }

        if command.no_cache {
            println!("{}", style("Cache disabled").yellow().bold());
        } else {
            println!(
                "{} {}",
                style("Cache miss").yellow().bold(),
                cache_path.display()
            );
        }

        let progress = ProgressBar::new(chunks.len() as u64);
        progress.set_style(
            ProgressStyle::with_template("{spinner:.cyan} chunk {pos}/{len} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );

        let mut segments = Vec::with_capacity(chunks.len());
        for (index, chunk) in chunks.iter().enumerate() {
            progress.set_message(format!("{} chars", chunk.chars().count()));
            debug!(index, chars = chunk.chars().count(), "generating TTS chunk");
            let mp3 = self.tts.synthesize(chunk, &command.options).await?;
            segments.push(mp3);
            progress.inc(1);
        }
        progress.finish_and_clear();

        let mp3 = concat_mp3_segments(&segments);
        if !command.no_cache {
            atomic_write(cache_path, &mp3)?;
            println!(
                "{} {}",
                style("Cached").green().bold(),
                cache_path.display()
            );
        }

        Ok((mp3, false))
    }
}

fn copy_to_output(mp3: &[u8], output: &Path) -> Result<()> {
    atomic_write(output, mp3)
}
