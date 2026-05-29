use async_trait::async_trait;
use async_openai::{
    config::{AzureConfig, OpenAIConfig},
    types::{
        CreateSpeechRequestArgs, SpeechModel, Voice as OpenAiVoice,
    },
    Client,
};

use crate::domain::{
    entities::{TtsModel, TtsOptions, Voice},
    ports::TtsProvider,
};

/// OpenAI TTS provider — supports both standard OpenAI and Azure OpenAI.
pub enum OpenAiTtsProvider {
    Standard(Client<OpenAIConfig>),
    Azure(Client<AzureConfig>),
}

impl OpenAiTtsProvider {
    /// Create a standard OpenAI TTS provider using the OPENAI_API_KEY env var.
    pub fn new_standard() -> Self {
        OpenAiTtsProvider::Standard(Client::new())
    }

    /// Create an Azure OpenAI TTS provider.
    pub fn new_azure(endpoint: &str, api_key: &str, deployment: &str) -> Self {
        let config = AzureConfig::new()
            .with_api_base(endpoint)
            .with_api_key(api_key)
            .with_deployment_id(deployment)
            .with_api_version("2024-02-01");
        OpenAiTtsProvider::Azure(Client::with_config(config))
    }
}

fn map_voice(voice: &Voice) -> OpenAiVoice {
    match voice {
        Voice::Alloy => OpenAiVoice::Alloy,
        Voice::Echo => OpenAiVoice::Echo,
        Voice::Fable => OpenAiVoice::Fable,
        Voice::Onyx => OpenAiVoice::Onyx,
        Voice::Nova => OpenAiVoice::Nova,
        Voice::Shimmer => OpenAiVoice::Shimmer,
    }
}

fn map_model(model: &TtsModel) -> SpeechModel {
    match model {
        TtsModel::Tts1 => SpeechModel::Tts1,
        TtsModel::Tts1Hd => SpeechModel::Tts1Hd,
    }
}

async fn do_synthesize<C>(
    client: &Client<C>,
    text: &str,
    options: &TtsOptions,
) -> anyhow::Result<Vec<u8>>
where
    C: async_openai::config::Config,
{
    let request = CreateSpeechRequestArgs::default()
        .input(text)
        .model(map_model(&options.model))
        .voice(map_voice(&options.voice))
        .speed(options.speed)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build speech request: {}", e))?;

    let response = client
        .audio()
        .speech(request)
        .await
        .map_err(|e| anyhow::anyhow!("OpenAI TTS API error: {}", e))?;

    Ok(response.bytes.to_vec())
}

#[async_trait]
impl TtsProvider for OpenAiTtsProvider {
    async fn synthesize(&self, text: &str, options: &TtsOptions) -> anyhow::Result<Vec<u8>> {
        match self {
            OpenAiTtsProvider::Standard(client) => do_synthesize(client, text, options).await,
            OpenAiTtsProvider::Azure(client) => do_synthesize(client, text, options).await,
        }
    }
}
