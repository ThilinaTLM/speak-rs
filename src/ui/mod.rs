use common::Message;
use iced::{
    Size, Subscription, alignment, color, theme, time,
    widget::{Column, button, column, container, row, text},
    window::Settings,
};

use std::time::{Duration, Instant};

use crate::{capture, whisper};

mod common;

pub struct SpeakUi {
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

impl Default for SpeakUi {
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

impl SpeakUi {
    pub fn view(&self) -> Column<Message> {
        column![
            container(column![
                container(
                    row![
                        button(text(if self.is_recording { "Stop" } else { "Start" }))
                            .on_press(Message::OnBtnRecord)
                            .padding([10, 30]),
                        text("Duration:").size(18).color(color!(120, 120, 120)),
                        text(format!(
                            "{:.2}",
                            self.recorder.get_duration().unwrap_or(0.0)
                        ))
                        .size(18)
                        .color(color!(60, 60, 60)),
                    ]
                    .height(50)
                    .align_y(alignment::Alignment::Center)
                    .spacing(20)
                ),
                container(text(&self.trans).size(20))
                    .width(iced::Length::Fill)
                    .height(iced::Length::Fill),
            ])
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .center_x(iced::Length::Fill)
            .padding(10)
        ]
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::OnBtnRecord => {
                self.is_recording = !self.is_recording;
                if self.is_recording {
                    self.recorder.start();
                } else {
                    self.recorder.pause();
                }
            }
            Message::Tick(tick) => {
                if self.is_recording && self.trans_updated_at.elapsed().as_secs() > 3 {
                    self.transcribe();
                }
                self.clock_updated_at = tick;
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(500)).map(Message::Tick)
    }

    fn theme(_: &SpeakUi) -> theme::Theme {
        theme::Theme::TokyoNight
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

        self.trans = transcription.combined.trim().to_string();
        self.trans_updated_at = Instant::now();

        let terminate_phrases = ["thats all", "that is all", "[blank audio]"];
        let cleaned_trans = self.trans.replace(".", "").replace("'", "").to_lowercase();
        if terminate_phrases
            .iter()
            .any(|phrase| cleaned_trans.ends_with(phrase))
        {
            self.is_recording = false;
            self.recorder.stop();
        }
    }
}

pub fn run() {
    let mut settings = Settings::default();
    settings.size = Size::new(600.0, 200.0);
    settings.resizable = false;
    settings.decorations = false;

    let _ = iced::application("Speak", SpeakUi::update, SpeakUi::view)
        .subscription(SpeakUi::subscription)
        .window(settings)
        .theme(SpeakUi::theme)
        .run();
}
