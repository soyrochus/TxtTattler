use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Top-level application configuration, loaded from a TOML file.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    pub voice: Option<String>,
    pub model: Option<String>,
    pub speed: Option<f32>,
    pub cache_dir: Option<PathBuf>,
    #[serde(default)]
    pub openai: OpenAiConfig,
    #[serde(default)]
    pub azure: AzureConfig,
}

/// OpenAI-specific configuration.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct OpenAiConfig {
    pub api_key: Option<String>,
}

/// Azure OpenAI-specific configuration.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct AzureConfig {
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
    pub deployment_id: Option<String>,
}

impl Config {
    /// Load configuration from `path`, or from the default location if `None`.
    ///
    /// If no config file is found, returns `Default::default()`.
    pub fn load(path: Option<&Path>) -> anyhow::Result<Self> {
        let config_path = match path {
            Some(p) => p.to_path_buf(),
            None => {
                let default = dirs::config_dir()
                    .unwrap_or_else(|| PathBuf::from(".config"))
                    .join("txttattler")
                    .join("txttattler.toml");
                if !default.exists() {
                    return Ok(Default::default());
                }
                default
            }
        };

        if !config_path.exists() {
            return Err(anyhow::anyhow!(
                "Config file '{}' not found",
                config_path.display()
            ));
        }

        let contents = std::fs::read_to_string(&config_path).map_err(|e| {
            anyhow::anyhow!("Failed to read config '{}': {}", config_path.display(), e)
        })?;

        let config: Config = toml::from_str(&contents).map_err(|e| {
            anyhow::anyhow!("Failed to parse config '{}': {}", config_path.display(), e)
        })?;

        Ok(config)
    }
}
