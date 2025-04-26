use anyhow::{Context, Result};
use log;

use super::MainWindow;
use crate::{capture, whisper};

pub fn handle_transcription_error(ui: &MainWindow, error: anyhow::Error) {
    log::error!("Transcription error: {}", error);
    ui.set_transcription(format!("Error: {}", error).into());
}

pub fn transcribe_audio(
    transcriber: &whisper::SimpleTranscriber,
    recorder: &capture::SimpleAudioCapture,
) -> Result<String> {
    let audio_data = recorder
        .get_audio_data()
        .context("Failed to get audio data")?;
    let sample_rate = recorder
        .get_sample_rate()
        .context("Failed to get sample rate")?;
    let channels = recorder
        .get_channels()
        .context("Failed to get channel count")?;
    let audio_duration = recorder
        .get_duration()
        .context("Failed to get audio duration")?;

    log::debug!(
        "audio summary:\n- duration: {}\n- sample rate: {}\n- channels: {}",
        audio_duration,
        sample_rate,
        channels
    );

    if audio_duration < 2.0 {
        log::warn!("audio duration is less than 2 seconds");
        return Ok(String::new());
    }

    let transcription = transcriber
        .transcribe(&whisper::InputAudio {
            data: &audio_data,
            sample_rate,
            channels,
        })
        .context("Failed to transcribe audio")?;

    log::debug!("transcription: {}", transcription.combined);
    Ok(transcription.combined.trim().to_string())
}
