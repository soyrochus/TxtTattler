use std::path::Path;
use std::sync::Arc;

use anyhow::Result;

use crate::application::text_to_speech::{SpeechCommand, TextToSpeechUseCase};
use crate::cli::Cli;
use crate::config::Settings;
use crate::domain::entities::{SpeechReport, TtsOptions};
use crate::infrastructure::audio::player::RodioAudioPlayer;
use crate::infrastructure::file_readers::FileReaderRegistry;
use crate::infrastructure::tts::openai::OpenAiTtsProvider;
use crate::utils::WhitespaceTextProcessor;

pub struct App {
    cli: Cli,
    settings: Settings,
    use_case: TextToSpeechUseCase,
}

impl App {
    pub fn new(settings: Settings, cli: Cli) -> Result<Self> {
        let readers = Arc::new(FileReaderRegistry::default_txt_only());
        let tts = Arc::new(OpenAiTtsProvider::new(&settings.openai, cli.azure)?);
        let player = Arc::new(RodioAudioPlayer);
        let processor = Arc::new(WhitespaceTextProcessor);
        let use_case = TextToSpeechUseCase::new(readers, tts, player, processor);

        Ok(Self {
            cli,
            settings,
            use_case,
        })
    }

    pub async fn run(&self, input: &Path) -> Result<SpeechReport> {
        self.use_case
            .execute(SpeechCommand {
                input: input.to_path_buf(),
                options: TtsOptions {
                    voice: self.settings.voice,
                    model: self.settings.model,
                    speed: self.settings.speed,
                },
                output: self.cli.output.clone(),
                no_play: self.cli.no_play,
                no_cache: self.cli.no_cache,
                refresh: self.cli.refresh,
                cache_dir: self.settings.cache_dir.clone(),
                verbose: self.cli.verbose,
            })
            .await
    }
}
