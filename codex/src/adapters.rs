use std::sync::Arc;

use anyhow::Result;

use crate::application::text_to_speech::{FileReaderRegistry, TextToSpeechUseCase};
use crate::config::AppConfig;
use crate::domain::entities::{TtsRequest, TtsResult};
use crate::domain::ports::TextProcessor;
use crate::infrastructure::audio::player::RodioAudioPlayer;
use crate::infrastructure::file_readers::txt::TxtFileReader;
use crate::infrastructure::tts::openai::OpenAiTtsProvider;
use crate::utils::WhitespaceNormalizer;

pub struct App {
    use_case: TextToSpeechUseCase,
}

impl App {
    pub fn new(config: AppConfig, force_azure: bool) -> Result<Self> {
        let mut readers = FileReaderRegistry::new();
        readers.register("txt", TxtFileReader);

        let processors: Vec<Arc<dyn TextProcessor>> = vec![Arc::new(WhitespaceNormalizer)];
        let tts = Arc::new(OpenAiTtsProvider::new(config, force_azure)?);
        let audio = Arc::new(RodioAudioPlayer);

        Ok(Self {
            use_case: TextToSpeechUseCase::new(readers, processors, tts, audio),
        })
    }

    pub async fn run(&self, request: TtsRequest) -> Result<TtsResult> {
        self.use_case.execute(request).await
    }
}
