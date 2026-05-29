use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use directories::{BaseDirs, ProjectDirs};
use serde::Deserialize;

use crate::cli::Cli;
use crate::domain::entities::{Model, Voice};

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct FileConfig {
    pub voice: Option<Voice>,
    pub model: Option<Model>,
    pub speed: Option<f32>,
    pub cache_dir: Option<PathBuf>,
    pub openai: Option<OpenAiConfigFile>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct OpenAiConfigFile {
    pub api_key: Option<String>,
    pub api_base: Option<String>,
    pub azure: Option<AzureConfigFile>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct AzureConfigFile {
    pub enabled: Option<bool>,
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
    pub deployment_id: Option<String>,
    pub api_version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub voice: Voice,
    pub model: Model,
    pub speed: f32,
    pub cache_dir: PathBuf,
    pub openai: OpenAiSettings,
}

#[derive(Debug, Clone, Default)]
pub struct OpenAiSettings {
    pub api_key: Option<String>,
    pub api_base: Option<String>,
    pub azure: AzureSettings,
}

#[derive(Debug, Clone)]
pub struct AzureSettings {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
    pub deployment_id: Option<String>,
    pub api_version: Option<String>,
}

impl Default for AzureSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: None,
            api_key: None,
            deployment_id: None,
            api_version: Some("2024-02-15-preview".to_string()),
        }
    }
}

impl Settings {
    pub fn load(config_path: Option<&Path>, cli: &Cli) -> Result<Self> {
        let _ = dotenvy::dotenv();

        let file_config = load_file_config(config_path)?;
        let openai_file = file_config.openai.unwrap_or_default();
        let azure_file = openai_file.azure.unwrap_or_default();

        let cache_dir = cli
            .cache_dir
            .clone()
            .or(file_config.cache_dir)
            .unwrap_or_else(default_cache_dir);

        Ok(Self {
            voice: cli.voice,
            model: cli.model,
            speed: cli.speed,
            cache_dir,
            openai: OpenAiSettings {
                api_key: env_or("OPENAI_API_KEY", openai_file.api_key),
                api_base: env_or("OPENAI_API_BASE", openai_file.api_base)
                    .or_else(|| env::var("OPENAI_BASE_URL").ok()),
                azure: AzureSettings {
                    enabled: cli.azure
                        || env_bool("AZURE_OPENAI", azure_file.enabled.unwrap_or(false)),
                    endpoint: env_or("AZURE_OPENAI_ENDPOINT", azure_file.endpoint),
                    api_key: env_or("AZURE_OPENAI_API_KEY", azure_file.api_key),
                    deployment_id: env_or("AZURE_OPENAI_DEPLOYMENT_ID", azure_file.deployment_id),
                    api_version: env_or("AZURE_OPENAI_API_VERSION", azure_file.api_version)
                        .or_else(|| Some("2024-02-15-preview".to_string())),
                },
            },
        })
    }
}

fn load_file_config(config_path: Option<&Path>) -> Result<FileConfig> {
    let path = config_path
        .map(PathBuf::from)
        .or_else(default_config_file)
        .filter(|path| path.exists());

    let Some(path) = path else {
        return Ok(FileConfig::default());
    };

    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))
}

fn default_cache_dir() -> PathBuf {
    ProjectDirs::from("io", "txttattler", "TxtTattler")
        .map(|dirs| dirs.cache_dir().to_path_buf())
        .or_else(|| BaseDirs::new().map(|dirs| dirs.home_dir().join(".cache/txttattler")))
        .unwrap_or_else(|| PathBuf::from(".txttattler-cache"))
}

fn default_config_file() -> Option<PathBuf> {
    ProjectDirs::from("io", "txttattler", "TxtTattler")
        .map(|dirs| dirs.config_dir().join("txttattler.toml"))
        .or_else(|| {
            BaseDirs::new().map(|dirs| dirs.home_dir().join(".config/txttattler/txttattler.toml"))
        })
}

fn env_or(name: &str, fallback: Option<String>) -> Option<String> {
    env::var(name)
        .ok()
        .filter(|value| !value.is_empty())
        .or(fallback)
}

fn env_bool(name: &str, fallback: bool) -> bool {
    match env::var(name).ok().as_deref() {
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("YES") => true,
        Some("0") | Some("false") | Some("FALSE") | Some("no") | Some("NO") => false,
        _ => fallback,
    }
}
