//! Configuration loader for TxtTattler.
//! Supports TOML file + environment variables + CLI overrides. Environment wins.

use crate::domain::entities::TtsOptions;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The root config structure (what lives in txttattler.toml)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TxtTattlerConfig {
    #[serde(default)]
    pub openai: OpenAiConfig,

    #[serde(default)]
    pub azure: AzureConfig,

    #[serde(default)]
    pub tts: TtsSection,

    /// Optional: custom text cleaning rules later
    #[serde(default)]
    pub processing: ProcessingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenAiConfig {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AzureConfig {
    pub api_key: Option<String>,
    #[serde(alias = "endpoint")]
    pub azure_endpoint: Option<String>,
    pub api_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsSection {
    #[serde(default = "default_voice")]
    pub voice: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_speed")]
    pub speed: f32,
}

impl Default for TtsSection {
    fn default() -> Self {
        Self {
            voice: default_voice(),
            model: default_model(),
            speed: default_speed(),
        }
    }
}

fn default_voice() -> String {
    "alloy".to_string()
}
fn default_model() -> String {
    "tts-1".to_string()
}
fn default_speed() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcessingConfig {
    /// Future: strip markdown, collapse newlines, etc.
    #[serde(default)]
    pub strip_markdown: bool,
}

/// Resolve the default config file location following XDG / platform conventions.
pub fn default_config_path() -> PathBuf {
    // ~/.config/txttattler/config.toml on Linux/macOS
    // %APPDATA%\txttattler\config.toml on Windows
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("txttattler")
        .join("config.toml")
}

/// Load config from disk if present. Missing file is fine (returns default).
pub fn load_config(path: Option<&Path>) -> Result<TxtTattlerConfig> {
    let path = path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(default_config_path);

    if !path.exists() {
        tracing::debug!("No config file at {} — using defaults + env", path.display());
        return Ok(TxtTattlerConfig::default());
    }

    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file at {}", path.display()))?;

    let cfg: TxtTattlerConfig = toml::from_str(&contents)
        .with_context(|| format!("Invalid TOML in {}", path.display()))?;

    tracing::info!("Loaded configuration from {}", path.display());
    Ok(cfg)
}

/// Final resolved settings after merging everything.
/// This is what the rest of the app sees.
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub tts_options: TtsOptions,
    pub openai_api_key: Option<String>,
    pub openai_base_url: Option<String>,
    pub azure_api_key: Option<String>,
    pub azure_endpoint: Option<String>,
    pub force_azure: bool,
}

impl ResolvedConfig {
    /// Merge: CLI > Env > TOML file > Hard defaults
    pub fn from_sources(cli: &crate::cli::Cli, file_cfg: &TxtTattlerConfig) -> Result<Self> {
        // NOTE: full env + file override logic can be expanded later.
        // For v1 we let CLI + direct env vars in the provider win for simplicity.
        // The file_cfg is available for future richer merging.
        let force_azure = cli.azure || std::env::var("TXTTATTLER_AZURE").is_ok();

        // CLI values (highest precedence)
        let voice = cli.voice.into();
        let model = cli.model.into();
        let speed = cli.speed;

        let tts_options = TtsOptions::new(voice, model, speed)?;

        Ok(Self {
            tts_options,
            openai_api_key: file_cfg.openai.api_key.clone().or_else(|| std::env::var("OPENAI_API_KEY").ok()),
            openai_base_url: file_cfg.openai.base_url.clone().or_else(|| std::env::var("OPENAI_BASE_URL").ok()),
            azure_api_key: file_cfg.azure.api_key.clone().or_else(|| std::env::var("AZURE_OPENAI_API_KEY").ok()),
            azure_endpoint: file_cfg
                .azure
                .azure_endpoint
                .clone()
                .or_else(|| std::env::var("AZURE_OPENAI_ENDPOINT").ok()),
            force_azure,
        })
    }

    pub fn is_azure_mode(&self) -> bool {
        self.force_azure || self.azure_endpoint.is_some()
    }
}

pub fn print_config_table(resolved: &ResolvedConfig, config_path: &Path) {
    use console::style;

    println!();
    println!("{}", style("📋 TXT TATTLER — EFFECTIVE CONFIGURATION").bold().cyan());
    println!("{}", style("─────────────────────────────────────────────").dim());

    println!("  Config file:     {}", style(config_path.display()).dim());
    println!("  Voice:           {}", style(resolved.tts_options.voice).yellow());
    println!("  Model:           {}", style(resolved.tts_options.model).yellow());
    println!("  Speed:           {}", style(format!("{:.2}x", resolved.tts_options.speed)).yellow());
    println!("  Azure mode:      {}", if resolved.is_azure_mode() {
        style("yes").red().bold()
    } else {
        style("no").green()
    });

    let has_openai = resolved.openai_api_key.is_some();
    let has_azure = resolved.azure_api_key.is_some() || resolved.azure_endpoint.is_some();

    println!("  OpenAI key:      {}", if has_openai { style("present").green() } else { style("MISSING").red().bold() });
    println!("  Azure key:       {}", if has_azure { style("present").green() } else { style("not set").dim() });

    println!();
}
