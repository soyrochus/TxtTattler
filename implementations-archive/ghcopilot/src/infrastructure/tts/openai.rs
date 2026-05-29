use anyhow::{Context, Result};
use async_openai::{
    Client,
    config::{AzureConfig, Config, OpenAIConfig},
    types::audio::{CreateSpeechRequestArgs, SpeechModel, Voice},
};
use async_trait::async_trait;

use crate::{
    config::{ProviderConfig, ResolvedConfig},
    domain::{
        entities::{SpeechModelName, TtsRequest, VoiceName},
        ports::TtsProvider,
    },
};

pub struct OpenAiTtsProvider {
    client: Client<Box<dyn Config>>,
}

impl OpenAiTtsProvider {
    pub fn from_config(config: &ResolvedConfig) -> Result<Self> {
        let provider: Box<dyn Config> = match &config.provider {
            ProviderConfig::OpenAi(openai) => {
                let mut openai_config = OpenAIConfig::new().with_api_key(openai.api_key.clone());
                if let Some(api_base) = &openai.api_base {
                    openai_config = openai_config.with_api_base(api_base.clone());
                }
                if let Some(org_id) = &openai.org_id {
                    openai_config = openai_config.with_org_id(org_id.clone());
                }
                if let Some(project_id) = &openai.project_id {
                    openai_config = openai_config.with_project_id(project_id.clone());
                }
                Box::new(openai_config)
            }
            ProviderConfig::Azure(azure) => Box::new(
                AzureConfig::new()
                    .with_api_base(azure.endpoint.clone())
                    .with_api_key(azure.api_key.clone())
                    .with_deployment_id(azure.deployment.clone())
                    .with_api_version(azure.api_version.clone()),
            ),
        };

        Ok(Self {
            client: Client::with_config(provider),
        })
    }
}

#[async_trait]
impl TtsProvider for OpenAiTtsProvider {
    async fn synthesize(&self, request: &TtsRequest) -> Result<Vec<u8>> {
        let openai_request = CreateSpeechRequestArgs::default()
            .input(request.text.clone())
            .voice(map_voice(request.voice))
            .model(map_model(request.model))
            .speed(request.speed)
            .build()
            .context("Failed to build the OpenAI speech request.")?;

        let response = self
            .client
            .audio()
            .speech()
            .create(openai_request)
            .await
            .context("Speech generation request failed.")?;

        Ok(response.bytes.to_vec())
    }
}

fn map_voice(voice: VoiceName) -> Voice {
    match voice {
        VoiceName::Alloy => Voice::Alloy,
        VoiceName::Echo => Voice::Echo,
        VoiceName::Fable => Voice::Fable,
        VoiceName::Onyx => Voice::Onyx,
        VoiceName::Nova => Voice::Nova,
        VoiceName::Shimmer => Voice::Shimmer,
    }
}

fn map_model(model: SpeechModelName) -> SpeechModel {
    match model {
        SpeechModelName::Tts1 => SpeechModel::Tts1,
        SpeechModelName::Tts1Hd => SpeechModel::Tts1Hd,
    }
}
