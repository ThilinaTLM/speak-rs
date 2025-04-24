#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use cpal::SampleFormat;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub struct SimpleAudioCapture {
    audio_data: Arc<Mutex<Option<Vec<f32>>>>,
    sample_rate: Arc<Mutex<Option<u32>>>,
    channels: Arc<Mutex<Option<usize>>>,
    is_recording: Arc<AtomicBool>,
    recording_thread: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl SimpleAudioCapture {
    pub fn new() -> Self {
        let audio_data = Arc::new(Mutex::new(Some(Vec::new())));
        let sample_rate = Arc::new(Mutex::new(None));
        let channels = Arc::new(Mutex::new(None));
        let is_recording = Arc::new(AtomicBool::new(false));
        let recording_thread = Arc::new(Mutex::new(None));

        Self {
            audio_data,
            sample_rate,
            channels,
            is_recording,
            recording_thread,
        }
    }

    pub fn start(&self) {
        if self.is_recording.load(Ordering::SeqCst) {
            return;
        }

        self.is_recording.store(true, Ordering::SeqCst);

        let audio_data_clone = self.audio_data.clone();
        let sample_rate_clone = self.sample_rate.clone();
        let channels_clone = self.channels.clone();
        let is_recording_clone = self.is_recording.clone();

        let handle = thread::spawn(move || {
            // Initialize audio input device
            let host = cpal::default_host();
            let device = match host.default_input_device() {
                Some(d) => d,
                None => {
                    eprintln!("No input device available");
                    is_recording_clone.store(false, Ordering::SeqCst);
                    return;
                }
            };

            let config = match device.default_input_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to get default input config: {}", e);
                    is_recording_clone.store(false, Ordering::SeqCst);
                    return;
                }
            };

            let sample_rate_value = config.sample_rate().0;
            let channels_value = config.channels() as usize;

            if let Ok(mut sr) = sample_rate_clone.lock() {
                *sr = Some(sample_rate_value);
            }
            if let Ok(mut ch) = channels_clone.lock() {
                *ch = Some(channels_value);
            }

            let err_fn = |err| eprintln!("An error occurred on the input audio stream: {}", err);

            let stream_result = match config.sample_format() {
                SampleFormat::F32 => {
                    let audio_clone = audio_data_clone.clone();
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[f32], _: &_| {
                            if let Ok(mut audio_buffer) = audio_clone.lock() {
                                if let Some(buffer) = audio_buffer.as_mut() {
                                    buffer.extend_from_slice(data);
                                }
                            }
                        },
                        err_fn,
                        None,
                    )
                }
                SampleFormat::I16 => {
                    let audio_clone = audio_data_clone.clone();
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[i16], _: &_| {
                            if let Ok(mut audio_buffer) = audio_clone.lock() {
                                if let Some(buffer) = audio_buffer.as_mut() {
                                    buffer.extend(data.iter().map(|&s| s as f32 / 32768.0));
                                }
                            }
                        },
                        err_fn,
                        None,
                    )
                }
                SampleFormat::U16 => {
                    let audio_clone = audio_data_clone.clone();
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[u16], _: &_| {
                            if let Ok(mut audio_buffer) = audio_clone.lock() {
                                if let Some(buffer) = audio_buffer.as_mut() {
                                    buffer
                                        .extend(data.iter().map(|&s| ((s as f32 / 32768.0) - 1.0)));
                                }
                            }
                        },
                        err_fn,
                        None,
                    )
                }
                _ => {
                    eprintln!("Unsupported sample format");
                    is_recording_clone.store(false, Ordering::SeqCst);
                    return;
                }
            };

            // The stream is kept locally in this thread
            let _stream = match stream_result {
                Ok(s) => {
                    if let Err(e) = s.play() {
                        eprintln!("Failed to play stream: {}", e);
                        is_recording_clone.store(false, Ordering::SeqCst);
                        return;
                    }
                    Some(s)
                }
                Err(e) => {
                    eprintln!("Failed to build input stream: {}", e);
                    is_recording_clone.store(false, Ordering::SeqCst);
                    return;
                }
            };

            // Keep thread alive while recording
            while is_recording_clone.load(Ordering::SeqCst) {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            // _stream is dropped when this thread ends
        });

        if let Ok(mut thread_guard) = self.recording_thread.lock() {
            *thread_guard = Some(handle);
        }
    }

    pub fn pause(&self) {
        self.is_recording.store(false, Ordering::SeqCst);

        if let Ok(mut thread_guard) = self.recording_thread.lock() {
            if let Some(handle) = thread_guard.take() {
                let _ = handle.join();
            }
        }
    }

    pub fn get_audio_data(&self) -> Option<Vec<f32>> {
        if let Ok(data) = self.audio_data.lock() {
            data.clone()
        } else {
            None
        }
    }

    pub fn get_sample_rate(&self) -> Option<u32> {
        if let Ok(rate) = self.sample_rate.lock() {
            *rate
        } else {
            None
        }
    }

    pub fn get_channels(&self) -> Option<usize> {
        if let Ok(ch) = self.channels.lock() {
            *ch
        } else {
            None
        }
    }

    pub fn get_duration(&self) -> Option<f32> {
        let audio_data = self.get_audio_data()?;
        let sample_rate = self.get_sample_rate()? as f32;
        let channels = self.get_channels()? as f32;

        let num_samples = audio_data.len() as f32;
        let duration = num_samples / (sample_rate * channels);

        Some(duration)
    }

    pub fn stop(&self) {
        self.is_recording.store(false, Ordering::SeqCst);

        if let Ok(mut thread_guard) = self.recording_thread.lock() {
            if let Some(handle) = thread_guard.take() {
                let _ = handle.join();
            }
        }

        if let Ok(mut data) = self.audio_data.lock() {
            *data = Some(Vec::new());
        }
    }
}
