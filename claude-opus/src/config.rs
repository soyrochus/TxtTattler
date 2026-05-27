//! Optional TOML configuration. Everything here is overridable by CLI flags and
//! environment variables — config is the quiet fallback, not the boss.

use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

/// Top-level config, deserialized from `txttattler.toml`.
///
/// Example:
/// ```toml
/// voice = "nova"
/// model = "tts-1-hd"
/// speed = 1.1
///
/// [openai]
/// api_key = "sk-..."
///
/// [azure]
/// endpoint = "https://my-resource.openai.azure.com"
/// deployment_id = "tts"
/// ```
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct OpenAiConfig {
    pub api_key: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct AzureConfig {
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
    pub deployment_id: Option<String>,
    pub api_version: Option<String>,
}

impl Config {
    /// Load config from an explicit path, or from the platform default
    /// location. A missing default file is *not* an error (we just use
    /// defaults); a missing *explicit* path is, because the user asked for it.
    pub fn load(explicit_path: Option<&Path>) -> anyhow::Result<Self> {
        match explicit_path {
            Some(path) => {
                let contents = std::fs::read_to_string(path).with_context(|| {
                    format!("could not read config file '{}'", path.display())
                })?;
                Self::from_toml(&contents, path)
            }
            None => {
                let Some(path) = Self::default_path() else {
                    return Ok(Config::default());
                };
                if !path.exists() {
                    return Ok(Config::default());
                }
                let contents = std::fs::read_to_string(&path).with_context(|| {
                    format!("could not read config file '{}'", path.display())
                })?;
                Self::from_toml(&contents, &path)
            }
        }
    }

    fn from_toml(contents: &str, path: &Path) -> anyhow::Result<Self> {
        toml::from_str(contents)
            .with_context(|| format!("could not parse config file '{}'", path.display()))
    }

    /// `~/.config/txttattler/txttattler.toml` (or the OS equivalent).
    pub fn default_path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("txttattler").join("txttattler.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parses_a_full_config() {
        let toml = r#"
            voice = "nova"
            model = "tts-1-hd"
            speed = 1.25
            cache_dir = "/tmp/tt"

            [openai]
            api_key = "sk-test"

            [azure]
            endpoint = "https://example.openai.azure.com"
            deployment_id = "tts"
        "#;
        let cfg = Config::from_toml(toml, &PathBuf::from("test.toml")).unwrap();
        assert_eq!(cfg.voice.as_deref(), Some("nova"));
        assert_eq!(cfg.speed, Some(1.25));
        assert_eq!(cfg.openai.api_key.as_deref(), Some("sk-test"));
        assert_eq!(
            cfg.azure.endpoint.as_deref(),
            Some("https://example.openai.azure.com")
        );
    }

    #[test]
    fn empty_config_is_all_defaults() {
        let cfg = Config::from_toml("", &PathBuf::from("test.toml")).unwrap();
        assert!(cfg.voice.is_none());
        assert!(cfg.openai.api_key.is_none());
    }

    #[test]
    fn missing_explicit_path_is_an_error() {
        let result = Config::load(Some(Path::new("/nope/does/not/exist.toml")));
        assert!(result.is_err());
    }
}
