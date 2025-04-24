use anyhow::Result;
use ctrlc;
use log::LevelFilter;
use owo_colors::OwoColorize;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

mod capture;
mod utils;
mod whisper;

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_module("whisper_rs::ggml_logging_hook", LevelFilter::Off)
        .init();

    // Setup Ctrl+C handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("Received Ctrl+C, stopping...");
        r.store(false, Ordering::SeqCst);
    })?;

    // Create and start recorder
    let recorder = capture::SimpleAudioCapture::new();
    recorder.start();
    println!("Recording audio input. Press Ctrl+C to stop...");

    // Keep running until Ctrl+C is received
    while running.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    println!("Stopping audio input...");

    // Stop recording
    recorder.pause();

    // Get recording data
    let audio_data = recorder.get_audio_data().unwrap_or_default();
    let sample_rate = recorder.get_sample_rate().unwrap_or(44100);
    let channels = recorder.get_channels().unwrap_or(1);

    // Calculate and print original audio duration
    let original_duration_sec = audio_data.len() as f32 / (sample_rate as f32 * channels as f32);
    println!(
        "Original audio duration: {:.2} seconds",
        original_duration_sec
    );

    // Resample
    let resampled_audio = utils::resample_to_16khz(&audio_data, sample_rate, channels)?;
    let resampled_duration_sec = resampled_audio.len() as f32 / (16000.0 * channels as f32);
    println!(
        "Resampled audio duration: {:.2} seconds",
        resampled_duration_sec
    );

    let transcriber = whisper::SimpleTranscriber::new();

    if resampled_duration_sec >= 1.0 {
        let output = transcriber.transcribe(&resampled_audio);
        println!("{}", "Transcription:".bright_green().bold());
        println!("{}", output.combined.bright_white());
    } else {
        println!("Audio too short for transcription (need at least 1 second)");
    }
    Ok(())
}
