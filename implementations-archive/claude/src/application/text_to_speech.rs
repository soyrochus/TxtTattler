use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::domain::entities::TtsOptions;
use crate::domain::ports::{AudioPlayer, FileReader, TextProcessor, TtsProvider};
use crate::utils::{chunk_text, compute_cache_key, concat_mp3_chunks};

/// Arguments for the text-to-speech use case.
pub struct TextToSpeechArgs {
    pub input_path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub cache_dir: Option<PathBuf>,
    pub options: TtsOptions,
    pub no_play: bool,
    pub no_cache: bool,
    pub refresh: bool,
    pub verbose: bool,
}

/// Orchestrates the full text-to-speech pipeline.
pub struct TextToSpeechUseCase {
    pub reader: Arc<dyn FileReader>,
    pub tts: Arc<dyn TtsProvider>,
    pub player: Arc<dyn AudioPlayer>,
    pub processor: Arc<dyn TextProcessor>,
}

impl TextToSpeechUseCase {
    pub fn new(
        reader: Arc<dyn FileReader>,
        tts: Arc<dyn TtsProvider>,
        player: Arc<dyn AudioPlayer>,
        processor: Arc<dyn TextProcessor>,
    ) -> Self {
        TextToSpeechUseCase {
            reader,
            tts,
            player,
            processor,
        }
    }

    pub async fn run(&self, args: TextToSpeechArgs) -> anyhow::Result<()> {
        // 1. Read the file
        println!(
            "{} Reading {}...",
            style("[1/4]").bold().dim(),
            style(args.input_path.display()).cyan()
        );
        let raw_text = self
            .reader
            .read(&args.input_path)
            .with_context(|| format!("Failed to read '{}'", args.input_path.display()))?;

        // 2. Process text
        let text = self.processor.process(&raw_text);
        if args.verbose {
            eprintln!("Text length after processing: {} chars", text.len());
        }

        // 3. Compute cache key
        let cache_key = compute_cache_key(
            &text,
            args.options.voice.as_str(),
            args.options.model.as_str(),
            args.options.speed,
        );

        // 4. Determine cache directory
        let cache_dir = args.cache_dir.clone().unwrap_or_else(|| {
            dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from(".cache"))
                .join("txttattler")
        });
        let cache_file = cache_dir.join(format!("{}.mp3", cache_key.0));

        // 5. Check for cached file
        let use_cache = !args.no_cache;
        let audio_bytes: Vec<u8> = if use_cache && !args.refresh && cache_file.exists() {
            println!(
                "{} Using cached audio ({})...",
                style("[2/4]").bold().dim(),
                style(cache_file.display()).dim()
            );
            fs::read(&cache_file)
                .with_context(|| format!("Failed to read cache file '{}'", cache_file.display()))?
        } else {
            // 6. Chunk text and synthesize
            let chunks = chunk_text(&text, 4096);
            let num_chunks = chunks.len();

            println!(
                "{} Synthesizing speech ({} chunk{})...",
                style("[2/4]").bold().dim(),
                num_chunks,
                if num_chunks == 1 { "" } else { "s" }
            );

            let pb = ProgressBar::new(num_chunks as u64);
            pb.set_style(
                ProgressStyle::with_template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} chunks",
                )
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars("#>-"),
            );

            let mut chunk_audio: Vec<Vec<u8>> = Vec::with_capacity(num_chunks);
            for chunk in &chunks {
                let audio = self
                    .tts
                    .synthesize(chunk, &args.options)
                    .await
                    .context("TTS synthesis failed")?;
                chunk_audio.push(audio);
                pb.inc(1);
            }
            pb.finish_with_message("done");

            concat_mp3_chunks(chunk_audio)
        };

        // 7. Write to cache (unless no_cache)
        if use_cache && (!cache_file.exists() || args.refresh) {
            println!(
                "{} Caching audio...",
                style("[3/4]").bold().dim()
            );
            fs::create_dir_all(&cache_dir)
                .with_context(|| format!("Failed to create cache dir '{}'", cache_dir.display()))?;
            fs::write(&cache_file, &audio_bytes)
                .with_context(|| format!("Failed to write cache file '{}'", cache_file.display()))?;
            if args.verbose {
                eprintln!("Cached to: {}", cache_file.display());
            }
        } else if !use_cache {
            println!(
                "{} Skipping cache (--no-cache).",
                style("[3/4]").bold().dim()
            );
        } else {
            println!(
                "{} Cache already up to date.",
                style("[3/4]").bold().dim()
            );
        }

        // 8. Copy to output path if provided
        if let Some(ref out) = args.output_path {
            fs::write(out, &audio_bytes)
                .with_context(|| format!("Failed to write output file '{}'", out.display()))?;
            println!(
                "    Saved audio to {}",
                style(out.display()).green()
            );
        }

        // 9. Play audio unless --no-play
        if !args.no_play {
            println!(
                "{} Playing audio...",
                style("[4/4]").bold().dim()
            );
            self.player
                .play(&audio_bytes)
                .context("Audio playback failed")?;
            println!("    {}", style("Done!").green().bold());
        } else {
            println!(
                "{} Playback skipped (--no-play).",
                style("[4/4]").bold().dim()
            );
        }

        Ok(())
    }
}
