use anyhow::{Context, Result};
use config::{Config, Environment, File};
use directories::ProjectDirs;
use std::path::PathBuf;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct WhisperConfig {
    pub model_path: PathBuf,
    pub use_gpu: bool,
    pub language: String,
    pub audio_context: i32,
    pub no_speech_threshold: f32,
    pub num_threads: i32,
}

impl Default for WhisperConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::from("models/ggml-small.en.bin"),
            use_gpu: true,
            language: "en".to_string(),
            audio_context: 768,
            no_speech_threshold: 0.5,
            num_threads: 2,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct BehaviorConfig {
    pub realtime_transcribe: bool,
    pub auto_copy: bool,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            realtime_transcribe: true,
            auto_copy: true,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct AppConfig {
    pub whisper: WhisperConfig,
    pub behavior: BehaviorConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            whisper: WhisperConfig::default(),
            behavior: BehaviorConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn new() -> Result<Self> {
        // Get the configuration directory
        let project_dirs = ProjectDirs::from("rs", "", "speak-rs")
            .context("Failed to determine project directories")?;

        let config_dir = project_dirs.config_dir();
        std::fs::create_dir_all(config_dir)?;
        let config_path = config_dir.join("config.toml");

        // Create default config if it doesn't exist
        if !config_path.exists() {
            let default_config = Self::default();
            let toml = toml::to_string_pretty(&default_config)?;
            std::fs::write(&config_path, toml)?;
        }

        // Build configuration with the following priority (highest to lowest):
        // 1. Environment variables (SPEAK_*)
        // 2. Configuration file
        // 3. Default values
        let config = Config::builder()
            // Start with default values
            .set_default("whisper.model_path", "models/ggml-small.en.bin")?
            .set_default("whisper.use_gpu", true)?
            .set_default("whisper.language", "en")?
            .set_default("whisper.audio_context", 768)?
            .set_default("whisper.no_speech_threshold", 0.5)?
            .set_default("whisper.num_threads", 2)?
            .set_default("behavior.realtime_transcribe", true)?
            .set_default("behavior.auto_copy", true)?
            // Add configuration file
            .add_source(File::with_name(config_path.to_str().unwrap()).required(false))
            // Add environment variables with prefix SPEAK_
            .add_source(Environment::with_prefix("SPEAK").separator("_"))
            .build()?;

        // Deserialize the configuration
        let app_config = config.try_deserialize()?;

        Ok(app_config)
    }

    pub fn get_config_path() -> Result<PathBuf> {
        let project_dirs = ProjectDirs::from("rs", "", "speak-rs")
            .context("Failed to determine project directories")?;
        Ok(project_dirs.config_dir().join("config.toml"))
    }
}
