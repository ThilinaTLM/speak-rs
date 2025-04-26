use std::{sync::Arc, time::Duration};

use anyhow::Result;
use arboard::Clipboard;
use log::LevelFilter;
use slint::{PhysicalPosition, Timer, TimerMode};

mod capture;
mod whisper;

slint::include_modules!();

fn main() -> Result<()> {
    whisper_rs::install_logging_hooks();
    env_logger::Builder::from_default_env()
        .filter_module("whisper_rs", LevelFilter::Off)
        .init();

    let ui = Arc::new(MainWindow::new()?);
    let recorder = Arc::new(capture::SimpleAudioCapture::new());
    let transcriber = Arc::new(whisper::SimpleTranscriber::new());
    let duration_timer = Arc::new(Timer::default());
    let transcription_timer = Arc::new(Timer::default());

    let _ui = ui.clone();
    let _recorder = recorder.clone();
    let duration_timer_fn = Arc::new(move || {
        let recording = _recorder.get_is_recording();
        if recording {
            if let Some(duration) = _recorder.get_duration() {
                let minutes = duration as u32 / 60;
                let seconds = duration as u32 % 60;
                _ui.set_duration_minutes(format!("{:02}", minutes).into());
                _ui.set_duration_seconds(format!("{:02}", seconds).into());
            }
        }
    });

    let _ui = ui.clone();
    let _recorder = recorder.clone();
    let _transcriber = transcriber.clone();
    let transcription_timer_fn = Arc::new(move || {
        let recording = _recorder.get_is_recording();
        if recording {
            let audio_data = _recorder.get_audio_data().unwrap_or_default();
            let sample_rate = _recorder.get_sample_rate().unwrap_or(44100);
            let channels = _recorder.get_channels().unwrap_or(1);
            let audio_duration = _recorder.get_duration().unwrap_or(0.0);

            if audio_duration < 2.0 {
                return;
            }

            let transcription = _transcriber
                .transcribe(&whisper::InputAudio {
                    data: &audio_data,
                    sample_rate,
                    channels,
                })
                .unwrap();

            _ui.set_transcription(transcription.combined.trim().to_string().into());
        }
    });

    let _recorder = recorder.clone();
    let _duration_timer = duration_timer.clone();
    let _transcription_timer = transcription_timer.clone();
    ui.on_close_button_clicked(move || {
        _recorder.stop();
        _duration_timer.stop();
        _transcription_timer.stop();
        std::process::exit(0);
    });

    let _recorder = recorder.clone();
    let _transcriber = transcriber.clone();
    let _ui = ui.clone();
    let _duration_timer = duration_timer.clone();
    let _transcription_timer = transcription_timer.clone();
    ui.on_record_button_clicked(move || {
        let recording = _ui.get_recording();
        if recording {
            _recorder.pause();
            _ui.set_recording(false);
            _duration_timer.stop();
            _transcription_timer.stop();

            _ui.set_transcription("transcribing...".into());
            let audio_data = _recorder.get_audio_data().unwrap_or_default();
            let sample_rate = _recorder.get_sample_rate().unwrap_or(44100);
            let channels = _recorder.get_channels().unwrap_or(1);
            let audio_duration = _recorder.get_duration().unwrap_or(0.0);

            if audio_duration < 2.0 {
                return;
            }

            let transcription = _transcriber
                .transcribe(&whisper::InputAudio {
                    data: &audio_data,
                    sample_rate,
                    channels,
                })
                .unwrap();

            _ui.set_transcription(transcription.combined.trim().to_string().into());
        } else {
            _ui.set_transcription("".into());
            _recorder.stop();
            _recorder.start();

            _ui.set_transcription("".into());
            _ui.set_recording(true);
            let _duration_timer_fn = duration_timer_fn.clone();
            _duration_timer.start(TimerMode::Repeated, Duration::from_millis(500), move || {
                _duration_timer_fn();
            });
            let _transcription_timer_fn = transcription_timer_fn.clone();
            _transcription_timer.start(
                TimerMode::Repeated,
                Duration::from_millis(3000),
                move || {
                    _transcription_timer_fn();
                },
            );
        }
    });

    let _ui = ui.clone();
    ui.on_copy_button_clicked(move || {
        let mut clipboard = Clipboard::new().unwrap();
        let transcription = _ui.get_transcription();
        if transcription.len() > 0 {
            let _ = clipboard.set_text(transcription.to_string());
        }
    });

    let _ui = ui.clone();
    ui.on_window_moved(move || {
        let mut pos = _ui.window().position();
        pos.x = 10;
        pos.y = 10;
        _ui.window().set_position(PhysicalPosition { x: 10, y: 10 });
    });

    ui.run()?;

    Ok(())
}
