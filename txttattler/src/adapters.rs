use std::sync::Arc;

use anyhow::{Result, bail};

use crate::{
    application::text_to_speech::{
        CollapseWhitespaceProcessor, ReduceBlankLinesProcessor, StripMarkdownProcessor,
        TextToSpeechService,
    },
    config::ResolvedConfig,
    domain::ports::{AudioPlayer, Reporter, TextProcessor, TtsProvider},
    infrastructure::{
        audio::player::RodioAudioPlayer,
        file_readers::{FileReaderRegistry, txt::TxtFileReader},
        tts::openai::OpenAiTtsProvider,
    },
    utils::ConsoleReporter,
};

pub struct App {
    config: ResolvedConfig,
    service: TextToSpeechService,
}

impl App {
    pub fn bootstrap(config: ResolvedConfig) -> Result<Self> {
        if !config.play_audio && config.output_path.is_none() && !config.cache_enabled {
            bail!(
                "Nothing to do: use playback, --output, or caching. Right now the tattler would gossip into /dev/null."
            );
        }

        let reporter: Arc<dyn Reporter> = Arc::new(ConsoleReporter::new(config.verbose));

        let mut registry = FileReaderRegistry::new();
        registry.register("txt", Arc::new(TxtFileReader::default()));

        let tts_provider: Arc<dyn TtsProvider> = Arc::new(OpenAiTtsProvider::from_config(&config)?);
        let audio_player: Arc<dyn AudioPlayer> = Arc::new(RodioAudioPlayer);

        let mut processors: Vec<Arc<dyn TextProcessor>> = vec![
            Arc::new(CollapseWhitespaceProcessor),
            Arc::new(ReduceBlankLinesProcessor),
        ];
        if config.strip_markdown {
            processors.push(Arc::new(StripMarkdownProcessor));
        }

        let service = TextToSpeechService::new(
            Arc::new(registry),
            tts_provider,
            audio_player,
            processors,
            reporter,
        );

        Ok(Self { config, service })
    }

    pub async fn run(self) -> Result<()> {
        self.service.run(&self.config).await?;
        Ok(())
    }
}
