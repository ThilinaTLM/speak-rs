// use iced::{
//     Background, Border, Element, Size, Subscription, Theme, alignment, border, color, theme, time,
//     widget::{button, column, container, row, svg, text},
//     window::Settings,
// };

// use std::time::{Duration, Instant};

// use crate::{capture, whisper};

// #[derive(Debug, Clone)]
// pub enum Message {
//     OnBtnRecord,
//     OnClose,
//     Tick(Instant),
// }

// pub struct SpeakUi {
//     // common
//     clock_updated_at: Instant,

//     // recording
//     is_recording: bool,
//     recorder: capture::SimpleAudioCapture,

//     // transcription
//     transcriber: whisper::SimpleTranscriber,
//     trans_updated_at: Instant,
//     trans: String,
//     trans_history: String,
//     audio_offset_ms: usize,
// }

// impl Default for SpeakUi {
//     fn default() -> Self {
//         let recorder = capture::SimpleAudioCapture::new();
//         let transcriber = whisper::SimpleTranscriber::new();
//         Self {
//             // common
//             clock_updated_at: Instant::now(),

//             // recording
//             is_recording: false,
//             recorder,

//             // transcription
//             transcriber,
//             trans_updated_at: Instant::now(),
//             trans: String::new(),
//             trans_history: String::new(),
//             audio_offset_ms: 0,
//         }
//     }
// }

// impl SpeakUi {
//     pub fn view(&self) -> Element<Message> {
//         container(column![
//             container(
//                 row![
//                     row![
//                         button(svg("assets/mic.svg").width(30).height(30))
//                             .on_press(Message::OnBtnRecord)
//                             .padding([10, 10])
//                             .style(|theme: &Theme, _| {
//                                 button::Style {
//                                     background: Some(Background::Color(if self.is_recording {
//                                         color!(255, 0, 0)
//                                     } else {
//                                         theme.palette().primary
//                                     })),
//                                     border: Border {
//                                         color: theme.palette().primary,
//                                         width: 1.0,
//                                         radius: border::Radius::new(5),
//                                     },
//                                     ..button::Style::default()
//                                 }
//                             }),
//                         text({
//                             let total_seconds = self.recorder.get_duration().unwrap_or(0.0) as u32;
//                             let minutes = total_seconds / 60;
//                             let seconds = total_seconds % 60;
//                             format!("{:02}:{:02}", minutes, seconds)
//                         })
//                         .size(18)
//                         .color(color!(60, 60, 60))
//                     ]
//                     .align_y(alignment::Alignment::Center)
//                     .spacing(10),
//                     button(svg("assets/x.svg").width(10).height(10))
//                         .on_press(Message::OnClose)
//                         .padding([10, 10])
//                         .style(|theme: &Theme, _| {
//                             button::Style {
//                                 background: Some(Background::Color(theme.palette().primary)),
//                                 border: Border {
//                                     color: theme.palette().primary,
//                                     width: 1.0,
//                                     radius: border::Radius::new(5),
//                                 },
//                                 ..button::Style::default()
//                             }
//                         }),
//                 ]
//                 .height(50)
//                 .align_y(alignment::Alignment::Center)
//                 .width(iced::Length::Fill)
//                 .spacing(20)
//             ).style(|_: &Theme| {
//                 container::Style {
//                     border: border::Border {
//                         color: color!(255, 0, 0),
//                         width: 1.0,
//                         radius: border::Radius::new(0),
//                     },
//                     ..container::Style::default()
//                 }
//             }),
//             container(row![
//                 text(&self.trans_history).size(20),
//                 text(&self.trans).size(20).color(color!(120, 120, 120)),
//             ]).style(|_: &Theme| {
//                 container::Style {
//                     border: border::Border {
//                         color: color!(0, 255, 0),
//                         width: 1.0,
//                         radius: border::Radius::new(0),
//                     },
//                     ..container::Style::default()
//                 }
//             }).height(iced::Length::Fill),
//         ])
//         .width(iced::Length::Fill)
//         .height(iced::Length::Fill)
//         .padding(10)
//         .style(|theme: &Theme| container::Style {
//             border: border::Border {
//                 color: theme.palette().primary,
//                 width: 2.0,
//                 radius: border::Radius::new(5),
//             },
//             ..container::Style::default()
//         })
//         .into()
//     }

//     pub fn update(&mut self, message: Message) {
//         match message {
//             Message::OnBtnRecord => {
//                 self.is_recording = !self.is_recording;
//                 if self.is_recording {
//                     self.recorder.start();
//                 } else {
//                     self.recorder.pause();
//                 }
//             }
//             Message::Tick(tick) => {
//                 if self.is_recording && self.trans_updated_at.elapsed().as_secs() > 3 {
//                     self.transcribe();
//                 }
//                 self.clock_updated_at = tick;
//             }
//             Message::OnClose => {
//                 self.is_recording = false;
//                 self.recorder.stop();
//             }
//         }
//     }

//     fn subscription(&self) -> Subscription<Message> {
//         time::every(Duration::from_millis(500)).map(Message::Tick)
//     }

//     fn theme(_: &SpeakUi) -> theme::Theme {
//         theme::Theme::TokyoNight
//     }

//     pub fn transcribe(&mut self) {
//         // Get current audio data and properties
//         let audio_data = self.recorder.get_audio_data().unwrap_or_default();
//         let sample_rate = self.recorder.get_sample_rate().unwrap_or(44100);
//         let channels = self.recorder.get_channels().unwrap_or(1);
//         let audio_duration = self.recorder.get_duration().unwrap_or(0.0);

//         if audio_duration < 3.0 {
//             return;
//         }

//         let audio_offset_index = self.audio_offset_ms * sample_rate as usize / 100;

//         // transcribe the audio data
//         let audio_window = &audio_data[audio_offset_index..];
//         let transcription = self
//             .transcriber
//             .transcribe(&whisper::InputAudio {
//                 data: audio_window,
//                 sample_rate,
//                 channels,
//             })
//             .unwrap();

//         self.trans = transcription.combined.trim().to_string();
//         self.trans_updated_at = Instant::now();

//         if transcription.segments.len() > 1 {
//             let one_before_last_segment = &transcription.segments[transcription.segments.len() - 2];
//             self.audio_offset_ms += one_before_last_segment.end;
//             self.trans_history = format!(
//                 "{} {}",
//                 self.trans_history,
//                 one_before_last_segment.text.trim()
//             );
//         }

//         let last_segment = &transcription.segments[transcription.segments.len() - 1];
//         self.trans = last_segment.text.trim().to_string();

//         for segment in &transcription.segments {
//             println!(
//                 "[{}:{}, {:.2}] {}",
//                 segment.start, segment.end, segment.confidence, segment.text
//             );
//         }
//     }
// }

// pub fn run() {
//     let mut settings = Settings::default();
//     settings.size = Size::new(700.0, 200.0);
//     settings.resizable = false;
//     settings.decorations = false;
//     settings.transparent = true;

//     let _ = iced::application("Speak", SpeakUi::update, SpeakUi::view)
//         .subscription(SpeakUi::subscription)
//         .window(settings)
//         .theme(SpeakUi::theme)
//         .run();
// }
