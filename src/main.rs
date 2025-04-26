use anyhow::Result;
use log::LevelFilter;

mod capture;
mod whisper;

slint::include_modules!();

fn main() -> Result<()> {
    whisper_rs::install_logging_hooks();
    env_logger::Builder::from_default_env()
        .filter_module("whisper_rs", LevelFilter::Off)
        .init();

    let ui = MainWindow::new()?;

    ui.run()?;

    Ok(())
}
