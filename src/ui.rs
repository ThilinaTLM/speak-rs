use iced::{
    Subscription, color, time,
    widget::{Column, button, column, row, text},
};

use std::time::{Duration, Instant};

use crate::{capture, whisper};

pub struct UiState {
    // common
    clock_updated_at: Instant,

    // recording
    is_recording: bool,
    recorder: capture::SimpleAudioCapture,

    // transcription
    transcriber: whisper::SimpleTranscriber,
    trans_updated_at: Instant,
    trans: String,
}

#[derive(Debug, Clone)]
pub enum UiEvent {
    OnBtnRecord,
    Tick(Instant),
}

impl Default for UiState {
    fn default() -> Self {
        let recorder = capture::SimpleAudioCapture::new();
        let transcriber = whisper::SimpleTranscriber::new();
        Self {
            // common
            clock_updated_at: Instant::now(),

            // recording
            is_recording: false,
            recorder,

            // transcription
            transcriber,
            trans_updated_at: Instant::now(),
            trans: String::new(),
        }
    }
}

impl UiState {
    pub fn view(&self) -> Column<UiEvent> {
        column![
            row![
                button(text("Record").color(if self.is_recording {
                    color!(255, 0, 0)
                } else {
                    color!(0, 255, 0)
                }))
                .on_press(UiEvent::OnBtnRecord)
                .padding(5),
                text("Duration: "),
                text(format!(
                    "{:.2}",
                    self.recorder.get_duration().unwrap_or(0.0)
                ))
            ],
            row![text(&self.trans),],
        ]
    }

    pub fn update(&mut self, message: UiEvent) {
        match message {
            UiEvent::OnBtnRecord => {
                self.is_recording = !self.is_recording;
                if self.is_recording {
                    self.recorder.start();
                } else {
                    self.recorder.pause();
                }
            }
            UiEvent::Tick(tick) => {
                if self.is_recording && self.trans_updated_at.elapsed().as_secs() > 3 {
                    self.transcribe();
                }
                self.clock_updated_at = tick;
            }
        }
    }

    pub fn transcribe(&mut self) {
        // Get current audio data and properties
        let audio_data = self.recorder.get_audio_data().unwrap_or_default();
        let sample_rate = self.recorder.get_sample_rate().unwrap_or(44100);
        let channels = self.recorder.get_channels().unwrap_or(1);
        let audio_duration = self.recorder.get_duration().unwrap_or(0.0);

        if audio_duration < 3.0 {
            return;
        }

        // transcribe the audio data
        let transcription = self
            .transcriber
            .transcribe(&whisper::InputAudio {
                data: &audio_data,
                sample_rate,
                channels,
            })
            .unwrap();

        self.trans = transcription.combined.clone();
        self.trans_updated_at = Instant::now();

        let terminate_phrases = ["thats all", "that is all", "[blank audio]"];
        let cleaned_trans = self.trans.replace(".", "").replace("'", "").to_lowercase();
        if terminate_phrases
            .iter()
            .any(|phrase| cleaned_trans.ends_with(phrase))
        {
            self.is_recording = false;
            self.recorder.pause();
        }
    }

    fn subscription(&self) -> Subscription<UiEvent> {
        time::every(Duration::from_millis(500)).map(UiEvent::Tick)
    }
}

pub fn run() {
    let _ = iced::application("Speak", UiState::update, UiState::view)
        .subscription(UiState::subscription)
        .run();
}
