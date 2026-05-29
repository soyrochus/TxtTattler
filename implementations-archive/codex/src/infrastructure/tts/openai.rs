use anyhow::{Context, Result};
use async_openai::{
    Client,
    config::{AzureConfig, Config, OpenAIConfig},
    types::audio::{CreateSpeechRequestArgs, SpeechModel, SpeechResponseFormat, Voice},
};
use async_trait::async_trait;
use tracing::debug;

use crate::config::AppConfig;
use crate::domain::{entities::TtsOptions, ports::TtsProvider};

type DynConfig = Box<dyn Config>;

pub struct OpenAiTtsProvider {
    client: Client<DynConfig>,
}

impl OpenAiTtsProvider {
    pub fn new(config: AppConfig, force_azure: bool) -> Result<Self> {
        let use_azure = force_azure || config.azure_endpoint().is_some();

        let client = if use_azure {
            let endpoint = config
                .azure_endpoint()
                .context("Azure mode needs AZURE_OPENAI_ENDPOINT or [azure].endpoint")?;
            let api_key = config
                .azure_api_key()
                .context("Azure mode needs AZURE_OPENAI_API_KEY or [azure].api_key")?;
            let deployment_id = config
                .azure_deployment_id()
                .context("Azure mode needs AZURE_OPENAI_DEPLOYMENT_ID or [azure].deployment_id")?;

            let azure = AzureConfig::new()
                .with_api_base(endpoint)
                .with_api_key(api_key)
                .with_api_version(config.azure_api_version())
                .with_deployment_id(deployment_id);

            Client::with_config(Box::new(azure) as DynConfig)
        } else {
            let api_key = config
                .openai_api_key()
                .context("OpenAI mode needs OPENAI_API_KEY or [openai].api_key")?;
            let mut openai = OpenAIConfig::new().with_api_key(api_key);

            if let Some(base_url) = config.openai_base_url() {
                openai = openai.with_api_base(base_url);
            }
            if let Some(org_id) = config.openai_org_id() {
                openai = openai.with_org_id(org_id);
            }
            if let Some(project_id) = config.openai_project_id() {
                openai = openai.with_project_id(project_id);
            }

            Client::with_config(Box::new(openai) as DynConfig)
        };

        Ok(Self { client })
    }
}

#[async_trait]
impl TtsProvider for OpenAiTtsProvider {
    async fn synthesize(&self, text: &str, options: &TtsOptions) -> Result<Vec<u8>> {
        debug!(
            chars = text.chars().count(),
            voice = options.voice.as_str(),
            model = options.model.as_str(),
            "requesting speech"
        );

        let request = CreateSpeechRequestArgs::default()
            .input(text)
            .model(to_openai_model(options.model.as_str()))
            .voice(to_openai_voice(options.voice.as_str()))
            .response_format(SpeechResponseFormat::Mp3)
            .speed(options.speed)
            .build()?;

        let response = self.client.audio().speech().create(request).await?;
        Ok(response.bytes.to_vec())
    }
}

fn to_openai_voice(voice: &str) -> Voice {
    match voice {
        "alloy" => Voice::Alloy,
        "echo" => Voice::Echo,
        "fable" => Voice::Fable,
        "onyx" => Voice::Onyx,
        "nova" => Voice::Nova,
        "shimmer" => Voice::Shimmer,
        other => Voice::Other(other.to_owned()),
    }
}

fn to_openai_model(model: &str) -> SpeechModel {
    match model {
        "tts-1" => SpeechModel::Tts1,
        "tts-1-hd" => SpeechModel::Tts1Hd,
        other => SpeechModel::Other(other.to_owned()),
    }
}
