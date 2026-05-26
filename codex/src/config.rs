use std::{env, fs, path::Path};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct AppConfig {
    pub openai: Option<OpenAiConfig>,
    pub azure: Option<AzureOpenAiConfig>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct OpenAiConfig {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub org_id: Option<String>,
    pub project_id: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct AzureOpenAiConfig {
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub deployment_id: Option<String>,
    pub api_version: Option<String>,
}

impl AppConfig {
    pub fn load(explicit_path: Option<&Path>, azure_forced: bool) -> Result<Self> {
        let default_path = default_config_path();
        let config_path = explicit_path.or(default_path.as_deref());

        let mut config = match config_path {
            Some(path) if path.exists() => {
                let raw = fs::read_to_string(path)
                    .with_context(|| format!("failed to read config file {}", path.display()))?;
                toml::from_str(&raw)
                    .with_context(|| format!("failed to parse config file {}", path.display()))?
            }
            _ => Self::default(),
        };

        config.apply_env_overrides();

        if azure_forced && config.azure_endpoint().is_none() {
            anyhow::bail!("Azure mode needs AZURE_OPENAI_ENDPOINT or [azure].endpoint in config");
        }

        Ok(config)
    }

    pub fn openai_api_key(&self) -> Option<&str> {
        self.openai.as_ref()?.api_key.as_deref()
    }

    pub fn openai_base_url(&self) -> Option<&str> {
        self.openai.as_ref()?.base_url.as_deref()
    }

    pub fn openai_org_id(&self) -> Option<&str> {
        self.openai.as_ref()?.org_id.as_deref()
    }

    pub fn openai_project_id(&self) -> Option<&str> {
        self.openai.as_ref()?.project_id.as_deref()
    }

    pub fn azure_api_key(&self) -> Option<&str> {
        self.azure.as_ref()?.api_key.as_deref()
    }

    pub fn azure_endpoint(&self) -> Option<&str> {
        self.azure.as_ref()?.endpoint.as_deref()
    }

    pub fn azure_deployment_id(&self) -> Option<&str> {
        self.azure.as_ref()?.deployment_id.as_deref()
    }

    pub fn azure_api_version(&self) -> &str {
        self.azure
            .as_ref()
            .and_then(|azure| azure.api_version.as_deref())
            .unwrap_or("2024-02-15-preview")
    }

    fn apply_env_overrides(&mut self) {
        let openai = self.openai.get_or_insert_with(OpenAiConfig::default);
        set_from_env(&mut openai.api_key, "OPENAI_API_KEY");
        set_from_env(&mut openai.base_url, "OPENAI_BASE_URL");
        set_from_env(&mut openai.org_id, "OPENAI_ORG_ID");
        set_from_env(&mut openai.project_id, "OPENAI_PROJECT_ID");

        let azure = self.azure.get_or_insert_with(AzureOpenAiConfig::default);
        set_from_env(&mut azure.endpoint, "AZURE_OPENAI_ENDPOINT");
        set_from_env(&mut azure.api_key, "AZURE_OPENAI_API_KEY");
        set_from_env(&mut azure.api_key, "AZURE_OPENAI_KEY");
        set_from_env(&mut azure.deployment_id, "AZURE_OPENAI_DEPLOYMENT");
        set_from_env(&mut azure.deployment_id, "AZURE_OPENAI_DEPLOYMENT_ID");
        set_from_env(&mut azure.api_version, "AZURE_OPENAI_API_VERSION");
    }
}

fn default_config_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|dir| dir.join("txttattler").join("txttattler.toml"))
}

fn set_from_env(target: &mut Option<String>, key: &str) {
    if let Ok(value) = env::var(key) {
        if !value.trim().is_empty() {
            *target = Some(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_toml() {
        let parsed: AppConfig = toml::from_str(
            r#"
            [openai]
            api_key = "sk-test"

            [azure]
            endpoint = "https://example.openai.azure.com"
            deployment_id = "tts"
            "#,
        )
        .unwrap();

        assert_eq!(parsed.openai_api_key(), Some("sk-test"));
        assert_eq!(
            parsed.azure_endpoint(),
            Some("https://example.openai.azure.com")
        );
    }
}
