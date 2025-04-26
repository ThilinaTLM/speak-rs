use std::sync::Arc;

use anyhow::Result;
use log::LevelFilter;

mod capture;
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

    // Initialize core components
    let recorder = Arc::new(capture::SimpleAudioCapture::new());
    let transcriber = Arc::new(whisper::SimpleTranscriber::new());

    // Initialize and run UI
    let app_ui = ui::AppUI::new(recorder, transcriber)?;
    app_ui.run()?;

    Ok(())
}
