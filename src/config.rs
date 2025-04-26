use std::path::PathBuf;

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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
    pub fn new() -> Self {
        // TODO: Load from config file if exists
        Self::default()
    }
}
