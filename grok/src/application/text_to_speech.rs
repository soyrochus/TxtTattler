//! The core use-case: TextToSpeechService implementation.
//! This is the heart of the tattler — it knows nothing about clap, files on disk, or HTTP.
//! It only knows about the three ports.

use crate::domain::entities::{ProcessedDocument, TtsOptions, TtsResult};
use crate::domain::ports::{
    AudioPlayer, FileReaderRegistry, TextToSpeechService, TtsProvider,
};
use crate::infrastructure::cache::Mp3Cache;
use anyhow::{Context, Result};
use async_trait::async_trait;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use std::time::Duration;

const OPENAI_MAX_CHARS_PER_CHUNK: usize = 3800; // safe margin under 4096

/// The concrete implementation of the gossip delivery service.
pub struct TextToSpeechOrchestrator {
    reader_registry: &'static FileReaderRegistry,
    tts_provider: Box<dyn TtsProvider>,
    audio_player: Box<dyn AudioPlayer>,
}

impl TextToSpeechOrchestrator {
    pub fn new(
        reader_registry: &'static FileReaderRegistry,
        tts_provider: Box<dyn TtsProvider>,
        audio_player: Box<dyn AudioPlayer>,
    ) -> Self {
        Self {
            reader_registry,
            tts_provider,
            audio_player,
        }
    }

    /// Load + clean a document using the appropriate registered reader.
    pub fn load_document(&self, path: &Path) -> Result<ProcessedDocument> {
        let reader = self
            .reader_registry
            .reader_for(path)
            .with_context(|| {
                format!(
                    "No reader registered for file extension '{:?}'. \
                     Currently supported: {:?}",
                    path.extension(),
                    self.reader_registry.supported_extensions()
                )
            })?;

        tracing::info!("Using {} to read {}", reader.name(), path.display());

        let text = reader.read(path)?;
        if text.trim().is_empty() {
            anyhow::bail!("The file {} appears to be empty. Nothing for the tattler to say!", path.display());
        }

        Ok(ProcessedDocument::new(path, text))
    }
}

#[async_trait]
impl TextToSpeechService for TextToSpeechOrchestrator {
    async fn tattle(
        &self,
        document: ProcessedDocument,
        options: TtsOptions,
        output_path: Option<PathBuf>,
        play_audio: bool,
        no_cache: bool,
        refresh: bool,
        cache_dir_override: Option<PathBuf>,
    ) -> Result<TtsResult> {
        use console::style;

        println!();
        println!(
            "{} {}",
            style("🗣️").bold(),
            style(format!("Tattling {} ...", document.original_filename)).cyan().bold()
        );
        println!(
            "   {} chars  •  {} words  •  voice: {}  •  model: {}",
            style(document.char_count).yellow(),
            style(document.word_count).yellow(),
            style(options.voice).green(),
            style(options.model).green()
        );

        // === NEW: MP3 Caching logic (Default MP3 Caching & Replay) ===
        let cache = if no_cache {
            None
        } else {
            Some(Mp3Cache::new(cache_dir_override.clone())?)
        };

        let cache_key = if let Some(ref c) = cache {
            let key = c.cache_key(
                &document.plain_text,
                options.voice.as_str(),
                options.model.as_str(),
                options.speed,
            );
            Some(key)
        } else {
            None
        };

        // Check for a happy cache hit (unless user said --refresh or --no-cache)
        if let (Some(ref c), Some(ref key)) = (&cache, &cache_key) {
            if !refresh {
                if let Some(cached_path) = c.get(key) {
                    println!(
                        "   {} {}",
                        style("💾").green().bold(),
                        style("Cache hit! Reusing previous gossip (no OpenAI call)").green()
                    );
                    if let Some(v) = &cache_dir_override {
                        tracing::debug!("Using custom cache dir: {}", v.display());
                    }

                    // Handle --output by copying the cached file (fast path)
                    let final_output_path = if let Some(ref out) = output_path {
                        if let Some(parent) = out.parent() {
                            std::fs::create_dir_all(parent).ok();
                        }
                        std::fs::copy(&cached_path, out)
                            .with_context(|| format!("Failed to copy cached MP3 to {}", out.display()))?;
                        println!(
                            "{} Copied cached gossip to {}",
                            style("💾").green(),
                            style(out.display()).bold()
                        );
                        Some(out.clone())
                    } else {
                        None
                    };

                    // Playback from the cached file when possible (more efficient)
                    if play_audio {
                        println!(
                            "{} {}",
                            style("🔊").bold(),
                            style("Playing from cache...").dim()
                        );
                        // Fall back to loading bytes if direct file play isn't supported by the player trait
                        let bytes = std::fs::read(&cached_path)?;
                        self.audio_player.play(&bytes).context("Cached playback failed")?;
                        println!("{}", style("✓ Finished playing cached gossip.").green());
                    }

                    return Ok(TtsResult {
                        total_chunks: 0, // we didn't synthesize anything
                        total_chars: document.char_count,
                        voice_used: options.voice,
                        model_used: options.model,
                        output_path: final_output_path,
                        duration_hint_secs: None,
                    });
                }
            } else {
                // --refresh: nuke the old entry so we always regenerate
                c.remove(key).ok();
                println!(
                    "   {} {}",
                    style("🔄").yellow(),
                    style("Refresh requested — regenerating and updating cache").yellow()
                );
            }
        }

        // === Normal synthesis path (cache miss or forced) ===
        if cache.is_some() {
            println!(
                "   {} {}",
                style("✨").dim(),
                style("Cache miss — calling the cloud gossips...").dim()
            );
        }

        let chunks = document.chunk(OPENAI_MAX_CHARS_PER_CHUNK);
        let total_chunks = chunks.len();

        if total_chunks > 1 {
            println!(
                "   {} Splitting into {} chunks (max ~{} chars each)",
                style("📦").dim(),
                style(total_chunks).bold(),
                OPENAI_MAX_CHARS_PER_CHUNK
            );
        }

        // Progress bar for synthesis phase
        let pb = ProgressBar::new(total_chunks as u64);
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} chunks {msg}",
            )
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏  "),
        );
        pb.enable_steady_tick(Duration::from_millis(80));

        let mut audio_segments: Vec<Vec<u8>> = Vec::with_capacity(total_chunks);
        let mut total_chars = 0usize;

        for (i, chunk) in chunks.iter().enumerate() {
            pb.set_message(format!("synthesizing #{}", i + 1));
            total_chars += chunk.char_count;

            let bytes = self
                .tts_provider
                .synthesize(chunk, &options)
                .await
                .with_context(|| format!("Failed while synthesizing chunk #{}", i + 1))?;

            audio_segments.push(bytes);
            pb.inc(1);
        }

        pb.finish_with_message("synthesis complete");

        // Combine once for caching + --output
        let mut combined_audio: Vec<u8> = Vec::new();
        for seg in &audio_segments {
            combined_audio.extend_from_slice(seg);
        }

        // === Write to cache if enabled ===
        if let (Some(ref c), Some(ref key)) = (&cache, &cache_key) {
            match c.put(key, &combined_audio) {
                Ok(cached_path) => {
                    println!(
                        "   {} Cached gossip to {}",
                        style("💾").green(),
                        style(cached_path.display()).dim()
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to write MP3 to cache: {}", e);
                }
            }
        }

        // Save combined output if requested (--output can also benefit from cache copy above)
        let final_output_path = if let Some(ref out) = output_path {
            // If we didn't already handle it via cache hit path
            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(out, &combined_audio)
                .with_context(|| format!("Failed to write output MP3 to {}", out.display()))?;

            println!(
                "{} Saved {} bytes of gossip to {}",
                style("💾").green(),
                style(combined_audio.len()).yellow(),
                style(out.display()).bold()
            );
            Some(out.clone())
        } else {
            None
        };

        // Playback
        if play_audio && !audio_segments.is_empty() {
            println!(
                "{} {}",
                style("🔊").bold(),
                style("Now playing through your speakers... (Ctrl-C to stop the tattler)").dim()
            );

            self.audio_player
                .play_chunks(&audio_segments)
                .context("Audio playback failed")?;

            println!("{}", style("✓ Finished playing. The tattler has spoken.").green());
        } else if play_audio && audio_segments.is_empty() {
            println!("{}", style("Nothing to play — the document was silent.").yellow());
        }

        Ok(TtsResult {
            total_chunks,
            total_chars,
            voice_used: options.voice,
            model_used: options.model,
            output_path: final_output_path,
            duration_hint_secs: None,
        })
    }
}
