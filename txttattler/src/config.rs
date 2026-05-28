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
    pub instructions: Option<String>,
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
    instructions: Option<String>,
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

        let voice_value = cli
            .voice
            .clone()
            .or_else(|| env::var("TXT_TATTLER_VOICE").ok())
            .or_else(|| file_config.voice.clone());
        let voice = parse_voice(voice_value.as_deref())?;
        let model_value = cli
            .model
            .clone()
            .or_else(|| env::var("TXT_TATTLER_MODEL").ok())
            .or_else(|| file_config.model.clone());
        let model = parse_model(model_value.as_deref())?;
        let speed = resolve_speed(
            cli.speed,
            env::var("TXT_TATTLER_SPEED").ok(),
            file_config.speed,
        )?;
        let instructions = cli
            .instructions
            .clone()
            .or_else(|| env::var("TXT_TATTLER_INSTRUCTIONS").ok())
            .or_else(|| file_config.instructions.clone());
        let cache_dir = cli
            .cache_dir
            .clone()
            .or_else(|| env::var("TXT_TATTLER_CACHE_DIR").ok().map(PathBuf::from))
            .or(file_config.cache_dir)
            .unwrap_or_else(|| project_dirs.cache_dir().to_path_buf());
        let verbose = cli.verbose
            || env_flag("TXT_TATTLER_VERBOSE")?.unwrap_or(file_config.verbose.unwrap_or(false));
        let strip_markdown = env_flag("TXT_TATTLER_STRIP_MARKDOWN")?
            .unwrap_or(file_config.strip_markdown.unwrap_or(false));

        let input_path = cli
            .file
            .clone()
            .ok_or_else(|| anyhow!("A file path is required unless --list-voices is used."))?;

        if cli.no_play && cli.output.is_none() && cli.no_cache {
            bail!("Nothing to do: enable playback, provide --output, or leave caching enabled.");
        }

        let use_azure = resolve_use_azure(cli.azure, file_config.azure)?;

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
            instructions,
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
    load_dotenv_from(&env::current_dir()?, Path::new(env!("CARGO_MANIFEST_DIR")))
}

fn load_dotenv_from(cwd: &Path, manifest_dir: &Path) -> Result<()> {
    let cwd_dotenv = cwd.join(".env");
    if cwd_dotenv.exists() {
        dotenvy::from_path(&cwd_dotenv)
            .with_context(|| format!("Failed to load {}", cwd_dotenv.display()))?;
        return Ok(());
    }

    let manifest_dotenv = manifest_dir.join(".env");
    if manifest_dotenv.exists() {
        dotenvy::from_path(&manifest_dotenv)
            .with_context(|| format!("Failed to load {}", manifest_dotenv.display()))?;
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

fn resolve_speed(
    cli_speed: Option<f32>,
    env_speed: Option<String>,
    config_speed: Option<f32>,
) -> Result<f32> {
    let speed = if let Some(speed) = cli_speed {
        speed
    } else if let Some(speed) = env_speed {
        speed.parse().with_context(|| {
            format!("TXT_TATTLER_SPEED must be a floating-point number, got '{speed}'.")
        })?
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

fn resolve_use_azure(cli_azure: bool, config_azure: Option<bool>) -> Result<bool> {
    if cli_azure {
        return Ok(true);
    }

    let configured_azure = env_flag("TXT_TATTLER_AZURE")?.or(config_azure);
    Ok(configured_azure.unwrap_or_else(|| {
        env::var("OPENAI_API_KEY").is_err() && env::var("AZURE_OPENAI_ENDPOINT").is_ok()
    }))
}

fn env_or_config(key: &str, config_value: Option<String>) -> Option<String> {
    env::var(key).ok().or(config_value)
}

fn env_flag(key: &str) -> Result<Option<bool>> {
    match env::var(key) {
        Ok(value) => parse_bool(&value).map(Some),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(err) => Err(anyhow!("Failed to read env var {key}: {err}")),
    }
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
    use super::{
        AzureFileConfig, FileConfig, OpenAiFileConfig, ProviderConfig, load_dotenv_from,
        parse_bool, resolve_azure, resolve_openai, resolve_speed, resolve_use_azure,
    };
    use crate::cli::Cli;
    use std::{
        env, fs,
        path::PathBuf,
        sync::{Mutex, OnceLock},
    };
    use tempfile::TempDir;

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    fn set_env(key: &str, value: &str) {
        unsafe { env::set_var(key, value) };
    }

    fn remove_env(key: &str) {
        unsafe { env::remove_var(key) };
    }

    fn clear_provider_env() {
        for key in [
            "OPENAI_API_KEY",
            "OPENAI_BASE_URL",
            "OPENAI_ORG_ID",
            "OPENAI_PROJECT_ID",
            "AZURE_OPENAI_ENDPOINT",
            "AZURE_OPENAI_API_KEY",
            "AZURE_OPENAI_DEPLOYMENT",
            "AZURE_OPENAI_API_VERSION",
            "TXT_TATTLER_AZURE",
            "TXT_TATTLER_INSTRUCTIONS",
            "TXT_TATTLER_MODEL",
            "TXT_TATTLER_VOICE",
            "TXT_TATTLER_SPEED",
            "TXT_TATTLER_CACHE_DIR",
            "TXT_TATTLER_VERBOSE",
            "TXT_TATTLER_STRIP_MARKDOWN",
        ] {
            remove_env(key);
        }
    }

    fn base_cli(tempdir: &TempDir, config_path: PathBuf) -> Cli {
        Cli {
            file: Some(tempdir.path().join("input.txt")),
            voice: None,
            model: None,
            speed: None,
            instructions: None,
            output: None,
            no_play: true,
            no_cache: false,
            refresh: false,
            cache_dir: None,
            azure: false,
            config: Some(config_path),
            list_voices: false,
            verbose: false,
        }
    }

    #[test]
    fn speed_prefers_config_when_cli_is_default() {
        let speed = resolve_speed(None, None, Some(1.5)).unwrap();
        assert_eq!(speed, 1.5);
    }

    #[test]
    fn explicit_default_speed_overrides_config() {
        let speed = resolve_speed(Some(1.0), None, Some(1.5)).unwrap();
        assert_eq!(speed, 1.0);
    }

    #[test]
    fn env_speed_overrides_config() {
        let speed = resolve_speed(None, Some("1.25".to_string()), Some(1.5)).unwrap();
        assert_eq!(speed, 1.25);
    }

    #[test]
    fn bool_parser_handles_yes() {
        assert!(parse_bool("yes").unwrap());
    }

    #[test]
    fn cwd_dotenv_loads_before_manifest_fallback() {
        let _guard = env_lock();
        remove_env("TXT_TATTLER_TEST_CWD_DOTENV");
        remove_env("TXT_TATTLER_TEST_MANIFEST_DOTENV");
        let cwd = TempDir::new().unwrap();
        let manifest = TempDir::new().unwrap();
        fs::write(cwd.path().join(".env"), "TXT_TATTLER_TEST_CWD_DOTENV=cwd\n").unwrap();
        fs::write(
            manifest.path().join(".env"),
            "TXT_TATTLER_TEST_MANIFEST_DOTENV=manifest\n",
        )
        .unwrap();

        load_dotenv_from(cwd.path(), manifest.path()).unwrap();

        assert_eq!(env::var("TXT_TATTLER_TEST_CWD_DOTENV").unwrap(), "cwd");
        assert!(env::var("TXT_TATTLER_TEST_MANIFEST_DOTENV").is_err());
        remove_env("TXT_TATTLER_TEST_CWD_DOTENV");
    }

    #[test]
    fn manifest_dotenv_loads_when_cwd_dotenv_is_absent() {
        let _guard = env_lock();
        remove_env("TXT_TATTLER_TEST_MANIFEST_ONLY_DOTENV");
        let cwd = TempDir::new().unwrap();
        let manifest = TempDir::new().unwrap();
        fs::write(
            manifest.path().join(".env"),
            "TXT_TATTLER_TEST_MANIFEST_ONLY_DOTENV=manifest\n",
        )
        .unwrap();

        load_dotenv_from(cwd.path(), manifest.path()).unwrap();

        assert_eq!(
            env::var("TXT_TATTLER_TEST_MANIFEST_ONLY_DOTENV").unwrap(),
            "manifest"
        );
        remove_env("TXT_TATTLER_TEST_MANIFEST_ONLY_DOTENV");
    }

    #[test]
    fn invalid_dotenv_error_mentions_path() {
        let cwd = TempDir::new().unwrap();
        let manifest = TempDir::new().unwrap();
        let dotenv = cwd.path().join(".env");
        fs::write(&dotenv, "BAD LINE\n").unwrap();

        let err = load_dotenv_from(cwd.path(), manifest.path()).unwrap_err();

        assert!(err.to_string().contains(&dotenv.display().to_string()));
    }

    #[test]
    fn openai_env_credentials_override_config() {
        let _guard = env_lock();
        clear_provider_env();
        set_env("OPENAI_API_KEY", "env-key");
        let cfg = OpenAiFileConfig {
            api_key: Some("config-key".to_string()),
            ..Default::default()
        };

        let resolved = resolve_openai(Some(&cfg)).unwrap();

        assert_eq!(resolved.api_key, "env-key");
        remove_env("OPENAI_API_KEY");
    }

    #[test]
    fn resolves_standard_openai_provider() {
        let _guard = env_lock();
        clear_provider_env();
        set_env("OPENAI_API_KEY", "test-key");

        let provider =
            if env::var("OPENAI_API_KEY").is_ok() && env::var("AZURE_OPENAI_ENDPOINT").is_err() {
                ProviderConfig::OpenAi(resolve_openai(None).unwrap())
            } else {
                unreachable!()
            };

        assert!(matches!(provider, ProviderConfig::OpenAi(_)));
        remove_env("OPENAI_API_KEY");
    }

    #[test]
    fn resolves_forced_azure_provider() {
        let _guard = env_lock();
        clear_provider_env();
        set_env("AZURE_OPENAI_ENDPOINT", "https://example.openai.azure.com");
        set_env("AZURE_OPENAI_API_KEY", "azure-key");
        set_env("AZURE_OPENAI_DEPLOYMENT", "tts");

        let provider = ProviderConfig::Azure(resolve_azure(None).unwrap());

        assert!(matches!(provider, ProviderConfig::Azure(_)));
        remove_env("AZURE_OPENAI_ENDPOINT");
        remove_env("AZURE_OPENAI_API_KEY");
        remove_env("AZURE_OPENAI_DEPLOYMENT");
    }

    #[test]
    fn azure_api_version_is_configurable() {
        let _guard = env_lock();
        clear_provider_env();
        set_env("AZURE_OPENAI_ENDPOINT", "https://example.openai.azure.com");
        set_env("AZURE_OPENAI_API_KEY", "azure-key");
        set_env("AZURE_OPENAI_DEPLOYMENT", "tts");
        set_env("AZURE_OPENAI_API_VERSION", "2025-01-01-preview");

        let resolved = resolve_azure(None).unwrap();

        assert_eq!(resolved.api_version, "2025-01-01-preview");
        remove_env("AZURE_OPENAI_ENDPOINT");
        remove_env("AZURE_OPENAI_API_KEY");
        remove_env("AZURE_OPENAI_DEPLOYMENT");
        remove_env("AZURE_OPENAI_API_VERSION");
    }

    #[test]
    fn azure_auto_detection_uses_endpoint_when_openai_key_absent() {
        let _guard = env_lock();
        clear_provider_env();
        set_env("AZURE_OPENAI_ENDPOINT", "https://example.openai.azure.com");

        assert!(resolve_use_azure(false, None).unwrap());
        clear_provider_env();
    }

    #[test]
    fn openai_key_prevents_accidental_azure_auto_detection() {
        let _guard = env_lock();
        clear_provider_env();
        set_env("OPENAI_API_KEY", "openai-key");
        set_env("AZURE_OPENAI_ENDPOINT", "https://example.openai.azure.com");

        assert!(!resolve_use_azure(false, None).unwrap());
        clear_provider_env();
    }

    #[test]
    fn env_azure_flag_overrides_config() {
        let _guard = env_lock();
        clear_provider_env();
        set_env("TXT_TATTLER_AZURE", "false");

        assert!(!resolve_use_azure(false, Some(true)).unwrap());
        clear_provider_env();
    }

    #[test]
    fn file_config_shape_keeps_provider_sections_optional() {
        let _cfg = FileConfig {
            instructions: Some("Speak in Dutch.".to_string()),
            openai: Some(OpenAiFileConfig::default()),
            azure_openai: Some(AzureFileConfig::default()),
            ..Default::default()
        };
    }

    #[test]
    fn env_instructions_set_resolved_config() {
        let _guard = env_lock();
        clear_provider_env();
        set_env("OPENAI_API_KEY", "test-key");
        set_env("TXT_TATTLER_INSTRUCTIONS", "env-value");
        let tempdir = TempDir::new().unwrap();
        let config_path = tempdir.path().join("txttattler.toml");
        fs::write(&config_path, "").unwrap();

        let resolved = super::ResolvedConfig::from_cli(&base_cli(&tempdir, config_path)).unwrap();

        assert_eq!(resolved.instructions, Some("env-value".to_string()));
        clear_provider_env();
    }

    #[test]
    fn cli_instructions_override_env_value() {
        let _guard = env_lock();
        clear_provider_env();
        set_env("OPENAI_API_KEY", "test-key");
        set_env("TXT_TATTLER_INSTRUCTIONS", "env-value");
        let tempdir = TempDir::new().unwrap();
        let config_path = tempdir.path().join("txttattler.toml");
        fs::write(&config_path, "").unwrap();
        let mut cli = base_cli(&tempdir, config_path);
        cli.instructions = Some("cli-value".to_string());

        let resolved = super::ResolvedConfig::from_cli(&cli).unwrap();

        assert_eq!(resolved.instructions, Some("cli-value".to_string()));
        clear_provider_env();
    }

    #[test]
    fn config_instructions_used_when_cli_and_env_absent() {
        let _guard = env_lock();
        clear_provider_env();
        set_env("OPENAI_API_KEY", "test-key");
        let tempdir = TempDir::new().unwrap();
        let config_path = tempdir.path().join("txttattler.toml");
        fs::write(&config_path, "instructions = \"config-value\"\n").unwrap();

        let resolved = super::ResolvedConfig::from_cli(&base_cli(&tempdir, config_path)).unwrap();

        assert_eq!(resolved.instructions, Some("config-value".to_string()));
        clear_provider_env();
    }

    #[test]
    fn no_instructions_source_resolves_to_none() {
        let _guard = env_lock();
        clear_provider_env();
        set_env("OPENAI_API_KEY", "test-key");
        let tempdir = TempDir::new().unwrap();
        let config_path = tempdir.path().join("txttattler.toml");
        fs::write(&config_path, "").unwrap();

        let resolved = super::ResolvedConfig::from_cli(&base_cli(&tempdir, config_path)).unwrap();

        assert_eq!(resolved.instructions, None);
        clear_provider_env();
    }

    #[test]
    fn missing_dotenv_is_ok() {
        let cwd = TempDir::new().unwrap();
        let manifest = TempDir::new().unwrap();
        load_dotenv_from(cwd.path(), manifest.path()).unwrap();
    }
}
