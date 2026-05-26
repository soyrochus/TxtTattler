use std::{
    env, fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{Context, Result, anyhow, bail};
use directories::ProjectDirs;
use serde::Deserialize;

use crate::{
    cli::Cli,
    domain::entities::{SpeechModelName, VoiceName},
};

const DEFAULT_AZURE_API_VERSION: &str = "2024-02-01";

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub input_path: PathBuf,
    pub voice: VoiceName,
    pub model: SpeechModelName,
    pub speed: f32,
    pub output_path: Option<PathBuf>,
    pub play_audio: bool,
    pub cache_enabled: bool,
    pub refresh: bool,
    pub cache_dir: PathBuf,
    pub verbose: bool,
    pub strip_markdown: bool,
    pub provider: ProviderConfig,
}

#[derive(Debug, Clone)]
pub enum ProviderConfig {
    OpenAi(OpenAiRuntimeConfig),
    Azure(AzureRuntimeConfig),
}

#[derive(Debug, Clone)]
pub struct OpenAiRuntimeConfig {
    pub api_key: String,
    pub api_base: Option<String>,
    pub org_id: Option<String>,
    pub project_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AzureRuntimeConfig {
    pub endpoint: String,
    pub api_key: String,
    pub deployment: String,
    pub api_version: String,
}

#[derive(Debug, Default, Deserialize)]
struct FileConfig {
    voice: Option<String>,
    model: Option<String>,
    speed: Option<f32>,
    cache_dir: Option<PathBuf>,
    azure: Option<bool>,
    verbose: Option<bool>,
    strip_markdown: Option<bool>,
    openai: Option<OpenAiFileConfig>,
    azure_openai: Option<AzureFileConfig>,
}

#[derive(Debug, Default, Clone, Deserialize)]
struct OpenAiFileConfig {
    api_key: Option<String>,
    api_base: Option<String>,
    org_id: Option<String>,
    project_id: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
struct AzureFileConfig {
    endpoint: Option<String>,
    api_key: Option<String>,
    deployment: Option<String>,
    api_version: Option<String>,
}

impl ResolvedConfig {
    pub fn from_cli(cli: &Cli) -> Result<Self> {
        load_dotenv()?;

        let project_dirs = project_dirs()?;
        let config_path = resolve_config_path(cli, &project_dirs)?;
        let file_config = if let Some(path) = &config_path {
            load_config_file(path)?
        } else {
            FileConfig::default()
        };

        let voice = parse_voice(cli.voice.as_deref().or(file_config.voice.as_deref()))?;
        let model = parse_model(cli.model.as_deref().or(file_config.model.as_deref()))?;
        let speed = resolve_speed(cli.speed, file_config.speed)?;
        let cache_dir = cli
            .cache_dir
            .clone()
            .or(file_config.cache_dir)
            .unwrap_or_else(|| project_dirs.cache_dir().to_path_buf());
        let verbose = cli.verbose || file_config.verbose.unwrap_or(false);
        let strip_markdown = file_config.strip_markdown.unwrap_or(false);

        let input_path = cli
            .file
            .clone()
            .ok_or_else(|| anyhow!("A file path is required unless --list-voices is used."))?;

        let use_azure = cli.azure
            || file_config.azure.unwrap_or(false)
            || env_flag("TXT_TATTLER_AZURE").unwrap_or(false)
            || (env::var("OPENAI_API_KEY").is_err() && env::var("AZURE_OPENAI_ENDPOINT").is_ok());

        let provider = if use_azure {
            ProviderConfig::Azure(resolve_azure(file_config.azure_openai.as_ref())?)
        } else {
            ProviderConfig::OpenAi(resolve_openai(file_config.openai.as_ref())?)
        };

        Ok(Self {
            input_path,
            voice,
            model,
            speed,
            output_path: cli.output.clone(),
            play_audio: !cli.no_play,
            cache_enabled: !cli.no_cache,
            refresh: cli.refresh,
            cache_dir,
            verbose,
            strip_markdown,
            provider,
        })
    }
}

fn load_dotenv() -> Result<()> {
    let dotenv_path = env::current_dir()?.join(".env");
    if dotenv_path.exists() {
        dotenvy::from_path(&dotenv_path)
            .with_context(|| format!("Failed to load {}", dotenv_path.display()))?;
    }
    Ok(())
}

fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", "txttattler")
        .ok_or_else(|| anyhow!("Unable to resolve platform config/cache directories."))
}

fn resolve_config_path(cli: &Cli, project_dirs: &ProjectDirs) -> Result<Option<PathBuf>> {
    if let Some(path) = &cli.config {
        if !path.exists() {
            bail!("Config file '{}' does not exist.", path.display());
        }
        return Ok(Some(path.clone()));
    }

    let default_path = project_dirs.config_dir().join("txttattler.toml");
    if default_path.exists() {
        Ok(Some(default_path))
    } else {
        Ok(None)
    }
}

fn load_config_file(path: &Path) -> Result<FileConfig> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file '{}'.", path.display()))?;
    toml::from_str(&raw)
        .with_context(|| format!("Failed to parse config file '{}'.", path.display()))
}

fn parse_voice(value: Option<&str>) -> Result<VoiceName> {
    let raw = value.unwrap_or("alloy");
    VoiceName::from_str(raw)
}

fn parse_model(value: Option<&str>) -> Result<SpeechModelName> {
    let raw = value.unwrap_or("tts-1");
    SpeechModelName::from_str(raw)
}

fn resolve_speed(cli_speed: f32, config_speed: Option<f32>) -> Result<f32> {
    let speed = if cli_speed != 1.0 {
        cli_speed
    } else {
        config_speed.unwrap_or(1.0)
    };

    if !(0.25..=4.0).contains(&speed) {
        bail!("Speed must be between 0.25 and 4.0.");
    }

    Ok(speed)
}

fn resolve_openai(file_config: Option<&OpenAiFileConfig>) -> Result<OpenAiRuntimeConfig> {
    let api_key = env_or_config(
        "OPENAI_API_KEY",
        file_config.and_then(|cfg| cfg.api_key.clone()),
    )
    .ok_or_else(|| {
        anyhow!("Missing OpenAI API key. Set OPENAI_API_KEY or configure [openai].api_key.")
    })?;

    Ok(OpenAiRuntimeConfig {
        api_key,
        api_base: env_or_config(
            "OPENAI_BASE_URL",
            file_config.and_then(|cfg| cfg.api_base.clone()),
        ),
        org_id: env_or_config(
            "OPENAI_ORG_ID",
            file_config.and_then(|cfg| cfg.org_id.clone()),
        ),
        project_id: env_or_config(
            "OPENAI_PROJECT_ID",
            file_config.and_then(|cfg| cfg.project_id.clone()),
        ),
    })
}

fn resolve_azure(file_config: Option<&AzureFileConfig>) -> Result<AzureRuntimeConfig> {
    let endpoint = env_or_config(
        "AZURE_OPENAI_ENDPOINT",
        file_config.and_then(|cfg| cfg.endpoint.clone()),
    )
    .ok_or_else(|| anyhow!("Missing Azure endpoint. Set AZURE_OPENAI_ENDPOINT or configure [azure_openai].endpoint."))?;

    let api_key = env_or_config(
        "AZURE_OPENAI_API_KEY",
        file_config.and_then(|cfg| cfg.api_key.clone()),
    )
    .ok_or_else(|| {
        anyhow!(
            "Missing Azure API key. Set AZURE_OPENAI_API_KEY or configure [azure_openai].api_key."
        )
    })?;

    let deployment = env_or_config(
        "AZURE_OPENAI_DEPLOYMENT",
        file_config.and_then(|cfg| cfg.deployment.clone()),
    )
    .ok_or_else(|| anyhow!("Missing Azure deployment. Set AZURE_OPENAI_DEPLOYMENT or configure [azure_openai].deployment."))?;

    let api_version = env_or_config(
        "AZURE_OPENAI_API_VERSION",
        file_config.and_then(|cfg| cfg.api_version.clone()),
    )
    .unwrap_or_else(|| DEFAULT_AZURE_API_VERSION.to_string());

    Ok(AzureRuntimeConfig {
        endpoint,
        api_key,
        deployment,
        api_version,
    })
}

fn env_or_config(key: &str, config_value: Option<String>) -> Option<String> {
    env::var(key).ok().or(config_value)
}

fn env_flag(key: &str) -> Result<bool> {
    match env::var(key) {
        Ok(value) => parse_bool(&value).map(Some),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(err) => Err(anyhow!("Failed to read env var {key}: {err}")),
    }
    .map(|value| value.unwrap_or(false))
}

fn parse_bool(value: &str) -> Result<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        other => bail!("Invalid boolean value '{other}'."),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_bool, resolve_speed};

    #[test]
    fn speed_prefers_config_when_cli_is_default() {
        let speed = resolve_speed(1.0, Some(1.5)).unwrap();
        assert_eq!(speed, 1.5);
    }

    #[test]
    fn bool_parser_handles_yes() {
        assert!(parse_bool("yes").unwrap());
    }
}
