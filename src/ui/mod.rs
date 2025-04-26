use anyhow::Result;
use arboard::Clipboard;
use i_slint_backend_winit::WinitWindowAccessor;
use log;
use slint::{BackendSelector, Timer, TimerMode};
use std::{sync::Arc, time::Duration};

use crate::{capture, config::BehaviorConfig, whisper};
use utils::{
    handle_transcription_error, is_endswith_pattern, remove_end_pattern, transcribe_audio,
};

mod utils;

slint::include_modules!();

pub struct AppUI {
    window: Arc<MainWindow>,
    recorder: Arc<capture::SimpleAudioCapture>,
    transcriber: Arc<whisper::SimpleTranscriber>,
    duration_timer: Arc<Timer>,
    transcription_timer: Arc<Timer>,
    behavior: BehaviorConfig,
}

impl AppUI {
    pub fn new(
        recorder: Arc<capture::SimpleAudioCapture>,
        transcriber: Arc<whisper::SimpleTranscriber>,
        behavior: BehaviorConfig,
    ) -> Result<Self> {
        let backend_selector = BackendSelector::new()
            .backend_name("winit".to_string())
            .renderer_name("skia".to_string());

        backend_selector.select()?;

        let window = Arc::new(MainWindow::new()?);
        let duration_timer = Arc::new(Timer::default());
        let transcription_timer = Arc::new(Timer::default());

        let ui = Self {
            window,
            recorder,
            transcriber,
            duration_timer,
            transcription_timer,
            behavior,
        };

        ui.setup_handlers();

        Ok(ui)
    }

    fn setup_handlers(&self) {
        let window = self.window.clone();
        let recorder = self.recorder.clone();
        let duration_timer = self.duration_timer.clone();
        let transcription_timer = self.transcription_timer.clone();
        let behavior = self.behavior.clone();

        // Duration timer function
        let duration_timer_fn = {
            let window = window.clone();
            let recorder = recorder.clone();
            Arc::new(move || {
                let recording = recorder.get_is_recording();
                if recording {
                    if let Some(duration) = recorder.get_duration() {
                        let minutes = duration as u32 / 60;
                        let seconds = duration as u32 % 60;
                        window.set_duration_minutes(format!("{:02}", minutes).into());
                        window.set_duration_seconds(format!("{:02}", seconds).into());
                    }
                }
            })
        };

        // Transcription timer function
        let transcription_timer_fn = {
            let window = window.clone();
            let recorder = recorder.clone();
            let transcriber = self.transcriber.clone();
            let behavior = behavior.clone();
            Arc::new(move || {
                if !behavior.realtime_transcribe {
                    return;
                }
                let recording = recorder.get_is_recording();
                if recording {
                    log::debug!("realtime transcribing audio");
                    match transcribe_audio(&transcriber, &recorder) {
                        Ok(text) => {
                            if !text.is_empty() {
                                if behavior.stop_phrase_enabled
                                    && is_endswith_pattern(&text, &behavior.stop_phrase_pattern)
                                {
                                    log::debug!("stopping phrase detected, stopping recording");
                                    window.invoke_record_button_clicked();
                                }

                                if window.get_recording() {
                                    window.set_transcription(text.into());
                                    log::debug!("ui updated with realtime transcription");
                                }
                            }
                        }
                        Err(err) => handle_transcription_error(&window, err),
                    }
                }
            })
        };

        // Close button handler
        {
            let recorder = recorder.clone();
            let duration_timer = duration_timer.clone();
            let transcription_timer = transcription_timer.clone();
            self.window.on_close_button_clicked(move || {
                recorder.clear();
                duration_timer.stop();
                transcription_timer.stop();
                std::process::exit(0);
            });
        }

        // Record button handler
        {
            let window = window.clone();
            let recorder = recorder.clone();
            let transcriber = self.transcriber.clone();
            let duration_timer = duration_timer.clone();
            let transcription_timer = transcription_timer.clone();
            let behavior = behavior.clone();

            self.window.on_record_button_clicked(move || {
                let recording = window.get_recording();
                if recording {
                    recorder.pause();
                    duration_timer.stop();
                    transcription_timer.stop();

                    log::debug!("final transcription");
                    match transcribe_audio(&transcriber, &recorder) {
                        Ok(text) => {
                            if !text.is_empty() {
                                if behavior.stop_phrase_enabled {
                                    if let Some(t) =
                                        remove_end_pattern(&text, &behavior.stop_phrase_pattern)
                                    {
                                        log::debug!(
                                            "transcribed text without stopping phrase: {}",
                                            t
                                        );
                                        window.set_transcription(t.clone().into());
                                        log::debug!("ui updated with transcription");
                                        if behavior.auto_copy {
                                            if let Ok(mut clipboard) = Clipboard::new() {
                                                let _ = clipboard.set_text(t.to_string());
                                            }
                                        }
                                    } else {
                                        window.set_transcription(text.clone().into());
                                        log::debug!("ui updated with transcription");
                                        if behavior.auto_copy {
                                            if let Ok(mut clipboard) = Clipboard::new() {
                                                let _ = clipboard.set_text(text.to_string());
                                            }
                                        }
                                    }
                                } else {
                                    window.set_transcription(text.clone().into());
                                    log::debug!("ui updated with transcription");
                                    if behavior.auto_copy {
                                        if let Ok(mut clipboard) = Clipboard::new() {
                                            let _ = clipboard.set_text(text.to_string());
                                        }
                                    }
                                }
                            }
                        }
                        Err(err) => handle_transcription_error(&window, err),
                    }
                    window.set_recording(false);
                } else {
                    recorder.clear();
                    recorder.start();
                    window.set_transcription("".into());
                    window.set_recording(true);

                    let duration_timer_fn = duration_timer_fn.clone();
                    duration_timer.start(
                        TimerMode::Repeated,
                        Duration::from_millis(500),
                        move || {
                            duration_timer_fn();
                        },
                    );

                    let transcription_timer_fn = transcription_timer_fn.clone();
                    transcription_timer.start(
                        TimerMode::Repeated,
                        Duration::from_millis(3000),
                        move || {
                            transcription_timer_fn();
                        },
                    );
                }
            });
        }

        // Copy button handler
        {
            let window = window.clone();
            self.window.on_copy_button_clicked(move || {
                if let Ok(mut clipboard) = Clipboard::new() {
                    let transcription = window.get_transcription();
                    if !transcription.is_empty() {
                        if let Err(err) = clipboard.set_text(transcription.to_string()) {
                            log::error!("Failed to copy to clipboard: {}", err);
                        }
                    }
                } else {
                    log::error!("Failed to access clipboard");
                }
            });
        }

        // Move window handler
        {
            let window = window.clone();
            self.window.on_set_window_dragging(move |_| {
                let _ = window.window().with_winit_window(|winit_win| {
                    let _ = winit_win.drag_window();
                    log::debug!("enabled window dragging");
                });
            });
        }
    }

    pub fn run(&self) -> Result<()> {
        self.window.run()?;
        Ok(())
    }
}
