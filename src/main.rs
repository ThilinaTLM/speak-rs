use anyhow::Result;
use log::LevelFilter;

mod capture;
mod ui;
mod whisper;

fn main() -> Result<()> {
    whisper_rs::install_logging_hooks();
    env_logger::Builder::from_default_env()
        .filter_module("whisper_rs", LevelFilter::Off)
        .init();

    ui::run();

    Ok(())
}
