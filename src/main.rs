use std::sync::Arc;

use anyhow::Result;
use log::{LevelFilter, info};

mod capture;
mod config;
mod ui;
mod whisper;

fn main() -> Result<()> {
    whisper_rs::install_logging_hooks();
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Info)
        .filter_module("whisper_rs", LevelFilter::Off)
        .format_level(true)
        .init();

    let config = config::AppConfig::new()?;
    info!(
        "Configuration loaded from: {}",
        config::AppConfig::get_config_path()?.display()
    );

    let recorder = Arc::new(capture::SimpleAudioCapture::new());
    let transcriber = Arc::new(whisper::SimpleTranscriber::new(config.whisper)?);

    let app_ui = ui::AppUI::new(recorder, transcriber, config.behavior)?;
    app_ui.run()?;

    Ok(())
}
