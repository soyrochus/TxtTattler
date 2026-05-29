//! The actual gossip engine: OpenAI's `/audio/speech` endpoint, via the
//! `async-openai` crate. Works against both api.openai.com and Azure OpenAI.

use async_openai::{
    Client,
    config::{AzureConfig, Config as OpenAiClientConfig, OpenAIConfig},
    types::{CreateSpeechRequestArgs, SpeechModel, Voice as OpenAiVoice},
};
use async_trait::async_trait;

use crate::domain::entities::{TtsModel, TtsOptions, Voice};
use crate::domain::ports::TtsProvider;

/// Default Azure API version if the user doesn't specify one.
const DEFAULT_AZURE_API_VERSION: &str = "2024-02-15-preview";

/// One TTS provider, two flavours: plain OpenAI or Azure OpenAI. The request
/// logic is identical — only the client configuration differs.
pub enum OpenAiTtsProvider {
    Standard(Client<OpenAIConfig>),
    Azure(Client<AzureConfig>),
}

impl OpenAiTtsProvider {
    /// Standard OpenAI. `api_key` overrides `OPENAI_API_KEY`; when `None`, the
    /// async-openai client reads the env var (populated from `.env` if present).
    pub fn standard(api_key: Option<String>) -> Self {
        let config = match api_key {
            Some(key) => OpenAIConfig::new().with_api_key(key),
            None => OpenAIConfig::default(),
        };
        OpenAiTtsProvider::Standard(Client::with_config(config))
    }

    /// Azure OpenAI. `endpoint` + `api_key` + `deployment` are required;
    /// `api_version` falls back to a sensible default.
    pub fn azure(
        endpoint: &str,
        api_key: &str,
        deployment: &str,
        api_version: Option<&str>,
    ) -> Self {
        let config = AzureConfig::new()
            .with_api_base(endpoint)
            .with_api_key(api_key)
            .with_deployment_id(deployment)
            .with_api_version(api_version.unwrap_or(DEFAULT_AZURE_API_VERSION));
        OpenAiTtsProvider::Azure(Client::with_config(config))
    }
}

fn map_voice(voice: Voice) -> OpenAiVoice {
    match voice {
        Voice::Alloy => OpenAiVoice::Alloy,
        Voice::Echo => OpenAiVoice::Echo,
        Voice::Fable => OpenAiVoice::Fable,
        Voice::Onyx => OpenAiVoice::Onyx,
        Voice::Nova => OpenAiVoice::Nova,
        Voice::Shimmer => OpenAiVoice::Shimmer,
    }
}

fn map_model(model: TtsModel) -> SpeechModel {
    match model {
        TtsModel::Tts1 => SpeechModel::Tts1,
        TtsModel::Tts1Hd => SpeechModel::Tts1Hd,
    }
}

/// Generic over the client config so we write the request once for both flavours.
async fn synthesize_with<C: OpenAiClientConfig>(
    client: &Client<C>,
    text: &str,
    options: &TtsOptions,
) -> anyhow::Result<Vec<u8>> {
    let request = CreateSpeechRequestArgs::default()
        .input(text)
        .voice(map_voice(options.voice))
        .model(map_model(options.model))
        .speed(options.speed)
        .build()
        .map_err(|e| anyhow::anyhow!("failed to build speech request: {e}"))?;

    let response = client
        .audio()
        .speech(request)
        .await
        .map_err(|e| anyhow::anyhow!("OpenAI TTS request failed: {e}"))?;

    Ok(response.bytes.to_vec())
}

#[async_trait]
impl TtsProvider for OpenAiTtsProvider {
    async fn synthesize(&self, text: &str, options: &TtsOptions) -> anyhow::Result<Vec<u8>> {
        match self {
            OpenAiTtsProvider::Standard(client) => synthesize_with(client, text, options).await,
            OpenAiTtsProvider::Azure(client) => synthesize_with(client, text, options).await,
        }
    }
}
