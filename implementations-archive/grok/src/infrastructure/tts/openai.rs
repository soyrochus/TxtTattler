//! OpenAI TTS adapter (supports both openai.com and Azure OpenAI).
//!
//! This is where the real magic happens: your text gets turned into the voice of a
//! slightly-too-enthusiastic friend who won't stop talking about your documents.

use crate::domain::entities::{TextChunk, TtsModel, TtsOptions, Voice};
use crate::domain::ports::TtsProvider;
use anyhow::{Context, Result};
use async_openai::{
    config::{AzureConfig, OpenAIConfig},
    types::{CreateSpeechRequest, SpeechModel, SpeechResponseFormat, Voice as OpenAIVoice},
    Client,
};
use async_trait::async_trait;

/// Internal enum so we don't need async-trait crate or boxing dynamic async.
enum TtsClient {
    OpenAI(Client<OpenAIConfig>),
    Azure(Client<AzureConfig>),
}

/// The one and only (for now) implementation that actually tattles to the cloud.
pub struct OpenAiTtsProvider {
    client: TtsClient,
    #[allow(dead_code)]
    is_azure: bool,
}

impl OpenAiTtsProvider {
    /// Create from environment (auto-detects Azure if AZURE_ vars are present or --azure was used).
    pub fn from_env(force_azure: bool) -> Result<Self> {
        let is_azure = force_azure || std::env::var("AZURE_OPENAI_ENDPOINT").is_ok();

        if is_azure {
            Self::new_azure()
        } else {
            Self::new_openai()
        }
    }

    pub fn new_openai() -> Result<Self> {
        if std::env::var("OPENAI_API_KEY").is_err() {
            anyhow::bail!(
                "No OPENAI_API_KEY found in environment.\n\
                 Set it with: export OPENAI_API_KEY=sk-...\n\
                 Or use --config to load from a txttattler.toml file.\n\
                 The tattler refuses to gossip without proper credentials."
            );
        }

        Ok(Self {
            client: TtsClient::OpenAI(Client::<OpenAIConfig>::new()),
            is_azure: false,
        })
    }

    pub fn new_azure() -> Result<Self> {
        let api_base = std::env::var("AZURE_OPENAI_ENDPOINT")
            .or_else(|_| std::env::var("AZURE_OPENAI_BASE_URL"))
            .context("AZURE_OPENAI_ENDPOINT required for Azure mode (e.g. https://yourname.openai.azure.com/)")?;

        let api_key = std::env::var("AZURE_OPENAI_API_KEY")
            .or_else(|_| std::env::var("AZURE_OPENAI_KEY"))
            .context("AZURE_OPENAI_API_KEY is required when using --azure")?;

        // For Azure TTS the deployment_id is usually the name you gave your TTS deployment.
        // If not provided, we try to use a sensible default (user can override via env).
        let deployment_id = std::env::var("AZURE_OPENAI_TTS_DEPLOYMENT")
            .or_else(|_| std::env::var("AZURE_OPENAI_DEPLOYMENT"))
            .unwrap_or_else(|_| "tts-1".to_string());  // common default name

        let mut azure_cfg = AzureConfig::new()
            .with_api_key(api_key)
            .with_api_base(api_base)
            .with_deployment_id(deployment_id);

        if let Ok(ver) = std::env::var("AZURE_OPENAI_API_VERSION") {
            azure_cfg = azure_cfg.with_api_version(ver);
        } else {
            // Azure OpenAI often requires a specific api-version for speech endpoints
            azure_cfg = azure_cfg.with_api_version("2024-02-15-preview");
        }

        Ok(Self {
            client: TtsClient::Azure(Client::with_config(azure_cfg)),
            is_azure: true,
        })
    }

    /// Create with explicit values from config file
    pub fn from_config(api_key: Option<String>, base_url: Option<String>, is_azure: bool) -> Result<Self> {
        if is_azure {
            let api_base = base_url.context("Azure config requires 'azure_endpoint'")?;
            let key = api_key.context("Azure config requires 'azure_api_key'")?;

            let deployment_id = std::env::var("AZURE_OPENAI_TTS_DEPLOYMENT")
                .unwrap_or_else(|_| "tts-1".to_string());

            let mut cfg = AzureConfig::new()
                .with_api_key(key)
                .with_api_base(api_base)
                .with_deployment_id(deployment_id);

            if let Ok(ver) = std::env::var("AZURE_OPENAI_API_VERSION") {
                cfg = cfg.with_api_version(ver);
            } else {
                cfg = cfg.with_api_version("2024-02-15-preview");
            }

            Ok(Self {
                client: TtsClient::Azure(Client::with_config(cfg)),
                is_azure: true,
            })
        } else {
            let mut cfg = OpenAIConfig::new();
            if let Some(key) = api_key {
                cfg = cfg.with_api_key(key);
            }
            if let Some(url) = base_url {
                // OpenAIConfig uses with_base_url? Check similar pattern
                // Actually OpenAIConfig builder also has with_api_base in recent
                cfg = cfg.with_api_base(url);  // may not exist, we'll catch at check time
            }
            Ok(Self {
                client: TtsClient::OpenAI(Client::with_config(cfg)),
                is_azure: false,
            })
        }
    }

    async fn do_speech(&self, req: CreateSpeechRequest) -> Result<Vec<u8>> {
        match &self.client {
            TtsClient::OpenAI(c) => {
                let resp = c
                    .audio()
                    .speech(req)
                    .await
                    .context("OpenAI TTS request failed — the cloud gossips refused to speak")?;
                Ok(resp.bytes.to_vec())
            }
            TtsClient::Azure(c) => {
                let resp = c
                    .audio()
                    .speech(req)
                    .await
                    .context("Azure OpenAI TTS failed — your corporate tattler is having a bad day")?;
                Ok(resp.bytes.to_vec())
            }
        }
    }
}

#[async_trait]
impl TtsProvider for OpenAiTtsProvider {
    fn name(&self) -> &'static str {
        if self.is_azure {
            "Azure OpenAI Corporate Tattler"
        } else {
            "OpenAI Cloud Gossip Network"
        }
    }

    async fn synthesize(&self, chunk: &TextChunk, options: &TtsOptions) -> Result<Vec<u8>> {
        if chunk.text.trim().is_empty() {
            return Ok(vec![]);
        }

        if chunk.char_count > 3800 {
            tracing::warn!(
                "Chunk {} has {} chars — close to OpenAI's limit. The tattler may truncate.",
                chunk.index,
                chunk.char_count
            );
        }

        let voice = match options.voice {
            Voice::Alloy => OpenAIVoice::Alloy,
            Voice::Echo => OpenAIVoice::Echo,
            Voice::Fable => OpenAIVoice::Fable,
            Voice::Onyx => OpenAIVoice::Onyx,
            Voice::Nova => OpenAIVoice::Nova,
            Voice::Shimmer => OpenAIVoice::Shimmer,
        };

        let model = match options.model {
            TtsModel::Tts1 => SpeechModel::Tts1,
            TtsModel::Tts1Hd => SpeechModel::Tts1Hd,
        };

        let request = CreateSpeechRequest {
            model,
            input: chunk.text.clone(),
            voice,
            response_format: Some(SpeechResponseFormat::Mp3),
            speed: Some(options.speed),
        };

        let start = std::time::Instant::now();
        let bytes = self
            .do_speech(request)
            .await
            .with_context(|| format!("TTS synthesis failed on chunk #{}", chunk.index))?;

        tracing::info!(
            "✓ Synthesized chunk #{} ({} chars) in {}ms → {} bytes of premium gossip",
            chunk.index + 1,
            chunk.char_count,
            start.elapsed().as_millis(),
            bytes.len()
        );

        Ok(bytes)
    }
}

