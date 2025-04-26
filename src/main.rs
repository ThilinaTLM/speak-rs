use std::sync::Arc;

use anyhow::Result;
use log::LevelFilter;

mod capture;
mod config;
mod ui;
mod whisper;

fn main() -> Result<()> {
    // Initialize logging
    whisper_rs::install_logging_hooks();
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Debug)
        .filter_module("whisper_rs", LevelFilter::Off)
        .format_level(true)
        .init();

    // Load configuration
    let config = config::AppConfig::new();

    // Initialize core components
    let recorder = Arc::new(capture::SimpleAudioCapture::new());
    let transcriber = Arc::new(whisper::SimpleTranscriber::new(config.whisper)?);

    // Initialize and run UI
    let app_ui = ui::AppUI::new(recorder, transcriber, config.behavior)?;
    app_ui.run()?;

    Ok(())
}
