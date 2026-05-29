use anyhow::{Context, Result};
use async_openai::Client;
use async_openai::config::{AzureConfig, Config, OpenAIConfig};
use async_openai::types::audio::{
    CreateSpeechRequestArgs, SpeechModel, SpeechResponseFormat, Voice as OpenAiVoice,
};
use async_trait::async_trait;

use crate::config::OpenAiSettings;
use crate::domain::entities::{Model, TtsOptions, Voice};
use crate::domain::ports::TtsProvider;

pub struct OpenAiTtsProvider {
    client: Client<Box<dyn Config>>,
}

impl OpenAiTtsProvider {
    pub fn new(settings: &OpenAiSettings, force_azure: bool) -> Result<Self> {
        let config: Box<dyn Config> = if force_azure || settings.azure.enabled {
            build_azure_config(settings)?
        } else {
            build_openai_config(settings)?
        };

        Ok(Self {
            client: Client::with_config(config),
        })
    }
}

#[async_trait]
impl TtsProvider for OpenAiTtsProvider {
    async fn synthesize(&self, text: &str, options: &TtsOptions) -> Result<Vec<u8>> {
        let request = CreateSpeechRequestArgs::default()
            .model(to_speech_model(options.model))
            .input(text)
            .voice(to_openai_voice(options.voice))
            .response_format(SpeechResponseFormat::Mp3)
            .speed(options.speed)
            .build()
            .context("failed to build OpenAI speech request")?;

        let response = self.client.audio().speech().create(request).await?;
        Ok(response.bytes.to_vec())
    }
}

fn build_openai_config(settings: &OpenAiSettings) -> Result<Box<dyn Config>> {
    let api_key = settings
        .api_key
        .clone()
        .context("OPENAI_API_KEY is required unless Azure mode is configured")?;
    let mut config = OpenAIConfig::new().with_api_key(api_key);
    if let Some(api_base) = &settings.api_base {
        config = config.with_api_base(api_base);
    }
    Ok(Box::new(config))
}

fn build_azure_config(settings: &OpenAiSettings) -> Result<Box<dyn Config>> {
    let azure = &settings.azure;
    let api_key = azure
        .api_key
        .clone()
        .context("AZURE_OPENAI_API_KEY is required for Azure mode")?;
    let endpoint = azure
        .endpoint
        .clone()
        .context("AZURE_OPENAI_ENDPOINT is required for Azure mode")?;
    let deployment_id = azure
        .deployment_id
        .clone()
        .context("AZURE_OPENAI_DEPLOYMENT_ID is required for Azure TTS")?;

    let mut config = AzureConfig::new()
        .with_api_key(api_key)
        .with_api_base(endpoint)
        .with_deployment_id(deployment_id);

    if let Some(api_version) = &azure.api_version {
        config = config.with_api_version(api_version);
    }

    Ok(Box::new(config))
}

fn to_speech_model(model: Model) -> SpeechModel {
    match model {
        Model::Tts1 => SpeechModel::Tts1,
        Model::Tts1Hd => SpeechModel::Tts1Hd,
    }
}

fn to_openai_voice(voice: Voice) -> OpenAiVoice {
    match voice {
        Voice::Alloy => OpenAiVoice::Alloy,
        Voice::Echo => OpenAiVoice::Echo,
        Voice::Fable => OpenAiVoice::Fable,
        Voice::Onyx => OpenAiVoice::Onyx,
        Voice::Nova => OpenAiVoice::Nova,
        Voice::Shimmer => OpenAiVoice::Shimmer,
    }
}
