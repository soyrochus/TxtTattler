use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use anyhow::{Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::fs;
use tracing::info;

use crate::domain::{
    entities::{TtsOptions, TtsRequest, TtsResult},
    ports::{AudioPlayer, FileReader, TextProcessor, TtsProvider},
};
use crate::utils::{cache_key, chunk_text};

const CACHE_VERSION: &str = "txttattler-cache-v1";

pub struct FileReaderRegistry {
    readers: HashMap<String, Arc<dyn FileReader>>,
}

impl FileReaderRegistry {
    pub fn new() -> Self {
        Self {
            readers: HashMap::new(),
        }
    }

    pub fn register<R>(&mut self, extension: impl Into<String>, reader: R)
    where
        R: FileReader + 'static,
    {
        self.readers
            .insert(extension.into().to_ascii_lowercase(), Arc::new(reader));
    }

    pub fn read(&self, path: &Path) -> Result<String> {
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase)
            .context("file has no extension; TxtTattler needs a clue")?;

        let reader = self
            .readers
            .get(&extension)
            .with_context(|| format!("unsupported file extension .{extension}"))?;

        reader.read(path)
    }
}

impl Default for FileReaderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TextToSpeechUseCase {
    readers: FileReaderRegistry,
    processors: Vec<Arc<dyn TextProcessor>>,
    tts: Arc<dyn TtsProvider>,
    audio: Arc<dyn AudioPlayer>,
}

impl TextToSpeechUseCase {
    pub fn new(
        readers: FileReaderRegistry,
        processors: Vec<Arc<dyn TextProcessor>>,
        tts: Arc<dyn TtsProvider>,
        audio: Arc<dyn AudioPlayer>,
    ) -> Self {
        Self {
            readers,
            processors,
            tts,
            audio,
        }
    }

    pub async fn execute(&self, request: TtsRequest) -> Result<TtsResult> {
        if request.file.as_os_str().is_empty() {
            anyhow::bail!("missing required <FILE>. Try: txttattler notes.txt");
        }

        let started = Instant::now();
        println!(
            "{} {}",
            style("Reading").cyan().bold(),
            request.file.display()
        );
        let mut text = self.readers.read(&request.file)?;
        let raw_chars = text.chars().count();

        println!("{}", style("Cleaning text").cyan().bold());
        for processor in &self.processors {
            text = processor.process(text)?;
        }

        if text.trim().is_empty() {
            anyhow::bail!("{} is empty after text cleanup", request.file.display());
        }

        let chunks = chunk_text(&text, 4096);
        let cleaned_chars = text.chars().count();
        println!(
            "{} {} chars -> {} chars, {} OpenAI TTS request(s)",
            style("Prepared").cyan().bold(),
            raw_chars,
            cleaned_chars,
            chunks.len()
        );
        println!(
            "{} voice={}, model={}, speed={}",
            style("Settings").cyan().bold(),
            request.voice.as_str(),
            request.model.as_str(),
            request.speed
        );
        info!(chunks = chunks.len(), "generating speech");

        let options = TtsOptions {
            voice: request.voice.clone(),
            model: request.model.clone(),
            speed: request.speed,
        };

        let cache_path = if request.use_cache {
            Some(resolve_cache_path(&request, &text)?)
        } else {
            None
        };

        let (mp3, cache_hit) = if let Some(path) = cache_path.as_ref() {
            if path.exists() && !request.refresh {
                println!(
                    "{} reusing {}",
                    style("Cache hit").green().bold(),
                    path.display()
                );
                (
                    fs::read(path)
                        .await
                        .with_context(|| format!("failed to read cached MP3 {}", path.display()))?,
                    true,
                )
            } else {
                if request.refresh && path.exists() {
                    println!(
                        "{} regenerating {}",
                        style("Cache refresh").yellow().bold(),
                        path.display()
                    );
                } else {
                    println!(
                        "{} will write {}",
                        style("Cache miss").yellow().bold(),
                        path.display()
                    );
                }

                let mp3 = self.generate_mp3(&chunks, &options).await?;
                write_mp3(path, &mp3).await?;
                println!("{} {}", style("Cached MP3").green().bold(), path.display());
                (mp3, false)
            }
        } else {
            println!("{}", style("Cache disabled for this run").yellow().bold());
            (self.generate_mp3(&chunks, &options).await?, false)
        };

        if let Some(path) = &request.output {
            write_mp3(path, &mp3).await?;
            println!("{} {}", style("Saved MP3").green().bold(), path.display());
        }

        if request.play {
            println!("{}", style("Playing audio").cyan().bold());
            self.audio.play(&mp3)?;
        } else {
            println!("{}", style("Playback skipped").yellow().bold());
        }

        println!(
            "{} finished in {:.1}s",
            style("Done").green().bold(),
            started.elapsed().as_secs_f32()
        );

        Ok(TtsResult {
            bytes: mp3.len(),
            chunks: chunks.len(),
            saved_to: request.output,
            cache_path,
            cache_hit,
        })
    }

    async fn generate_mp3(&self, chunks: &[String], options: &TtsOptions) -> Result<Vec<u8>> {
        println!(
            "{} generating {} chunk(s)",
            style("OpenAI TTS").cyan().bold(),
            chunks.len()
        );

        let progress = ProgressBar::new(chunks.len() as u64);
        progress.set_style(
            ProgressStyle::with_template("{spinner:.green} {msg} [{bar:30.cyan/blue}] {pos}/{len}")
                .unwrap()
                .progress_chars("#>-"),
        );
        progress.set_message("Generating speech");

        let mut mp3 = Vec::new();
        for (index, chunk) in chunks.iter().enumerate() {
            progress.set_message(format!("Generating speech chunk {}", index + 1));
            let segment = self.tts.synthesize(chunk, options).await?;
            mp3.extend_from_slice(&segment);
            progress.inc(1);
        }
        progress.finish_and_clear();

        Ok(mp3)
    }
}

fn resolve_cache_path(request: &TtsRequest, text: &str) -> Result<PathBuf> {
    let cache_dir = match &request.cache_dir {
        Some(path) => path.clone(),
        None => dirs::cache_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("txttattler"),
    };

    let speed = format!("{:.3}", request.speed);
    let key = cache_key(&[
        CACHE_VERSION,
        text,
        request.voice.as_str(),
        request.model.as_str(),
        &speed,
    ]);

    Ok(cache_dir.join(format!("{key}.mp3")))
}

async fn write_mp3(path: &Path, mp3: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, mp3)
        .await
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use async_trait::async_trait;

    use super::*;
    use crate::domain::{
        entities::{SpeechModel, Voice},
        ports::TextProcessor,
    };
    use crate::infrastructure::file_readers::txt::TxtFileReader;
    use crate::utils::WhitespaceNormalizer;

    struct CountingTts {
        calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl TtsProvider for CountingTts {
        async fn synthesize(&self, _text: &str, _options: &TtsOptions) -> Result<Vec<u8>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(b"fake mp3 bytes".to_vec())
        }
    }

    struct SilentAudio;

    impl AudioPlayer for SilentAudio {
        fn play(&self, _mp3_bytes: &[u8]) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn reuses_cached_mp3_for_same_text_and_options() {
        let temp = tempfile::tempdir().unwrap();
        let input = temp.path().join("note.txt");
        std::fs::write(&input, "the secret meeting is at noon").unwrap();

        let mut readers = FileReaderRegistry::new();
        readers.register("txt", TxtFileReader);

        let calls = Arc::new(AtomicUsize::new(0));
        let processors: Vec<Arc<dyn TextProcessor>> = vec![Arc::new(WhitespaceNormalizer)];
        let use_case = TextToSpeechUseCase::new(
            readers,
            processors,
            Arc::new(CountingTts {
                calls: calls.clone(),
            }),
            Arc::new(SilentAudio),
        );

        let request = TtsRequest {
            file: input,
            voice: Voice::Alloy,
            model: SpeechModel::Tts1,
            speed: 1.0,
            output: None,
            play: false,
            use_cache: true,
            refresh: false,
            cache_dir: Some(temp.path().join("cache")),
        };

        let first = use_case.execute(request.clone()).await.unwrap();
        let second = use_case.execute(request).await.unwrap();

        assert!(!first.cache_hit);
        assert!(second.cache_hit);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
