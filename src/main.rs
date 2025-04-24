use anyhow::Result;
use cpal::SampleFormat;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ctrlc;
use owo_colors::OwoColorize;
use rubato::{Resampler, SincFixedIn, SincInterpolationType, WindowFunction};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

fn resample_to_16khz(audio_data: &[f32], sample_rate: u32, channels: usize) -> Result<Vec<f32>> {
    if sample_rate == 16000 {
        return Ok(audio_data.to_vec());
    }

    // Print diagnostic information
    println!(
        "Resampling from {} Hz to 16000 Hz, channels: {}",
        sample_rate, channels
    );
    println!("Input samples: {}", audio_data.len());

    // Setup resampler
    let params = rubato::SincInterpolationParameters {
        sinc_len: 128,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };

    // Prepare input as channel separated data
    let frames = audio_data.len() / channels;
    let mut input_frames = vec![Vec::with_capacity(frames); channels];

    for frame in 0..frames {
        for ch in 0..channels {
            input_frames[ch].push(audio_data[frame * channels + ch]);
        }
    }

    // Create resampler
    let mut resampler = SincFixedIn::<f32>::new(
        16000 as f64 / sample_rate as f64,
        2.0,
        params,
        frames, // Use the full length as chunk size
        channels,
    )?;

    // Process all frames at once
    let resampled_channels = resampler.process(&input_frames, None)?;

    // Get delay information
    let delay = resampler.output_delay();

    // Calculate expected output length
    let expected_len = (frames as f64 * 16000 as f64 / sample_rate as f64) as usize;

    // Interleave the output back
    let mut output = Vec::with_capacity(expected_len * channels);
    for frame_idx in delay..resampled_channels[0].len().min(delay + expected_len) {
        for ch in 0..channels {
            output.push(resampled_channels[ch][frame_idx]);
        }
    }

    println!("Output samples: {}", output.len());
    Ok(output)
}

fn transcribe(audio_data: &[f32]) -> String {
    let path_to_model = "models/ggml-small.en.bin";
    let mut ctx_params = WhisperContextParameters::default();
    ctx_params.use_gpu(true);
    let ctx =
        WhisperContext::new_with_params(&path_to_model, ctx_params).expect("failed to load model");

    // Convert to mono if needed
    let mono_audio = whisper_rs::convert_stereo_to_mono_audio(audio_data)
        .expect("failed to convert audio to mono");

    // Create params with appropriate configuration
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some("en"));
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(true);

    // Run the model
    let mut state = ctx.create_state().expect("failed to create state");
    state
        .full(params, &mono_audio[..])
        .expect("failed to run model");

    // Fetch the results
    let num_segments = state
        .full_n_segments()
        .expect("failed to get number of segments");

    let mut transcription = String::new();
    for i in 0..num_segments {
        let segment = state
            .full_get_segment_text(i)
            .expect("failed to get segment");
        let start_timestamp = state
            .full_get_segment_t0(i)
            .expect("failed to get segment start timestamp");
        let end_timestamp = state
            .full_get_segment_t1(i)
            .expect("failed to get segment end timestamp");

        // Add formatted segment to transcription
        transcription.push_str(&format!(
            "[{} - {}]: {}\n",
            start_timestamp, end_timestamp, segment
        ));

        // Also print for convenience
        println!("[{} - {}]: {}", start_timestamp, end_timestamp, segment);
    }

    transcription
}

fn main() -> Result<()> {
    // Setup Ctrl+C handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("Received Ctrl+C, stopping...");
        r.store(false, Ordering::SeqCst);
    })?;

    // Initialize audio input device
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .expect("No input device available");
    let config = device
        .default_input_config()
        .expect("Failed to get default input config");
    let sample_rate = config.sample_rate().0;
    let channels = config.channels() as usize;

    println!("Input device: {}", device.name()?);
    println!(
        "Sample format: {:?}, sample rate: {}, channels: {}",
        config.sample_format(),
        sample_rate,
        channels
    );

    // Build input stream
    let err_fn = |err| eprintln!("An error occurred on the input audio stream: {}", err);
    let audio_data: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));

    let stream = match config.sample_format() {
        SampleFormat::F32 => {
            let audio_data_clone = audio_data.clone();
            device.build_input_stream(
                &config.into(),
                move |data: &[f32], _: &_| {
                    let len = data.len();
                    if len > 0 {
                        if let Ok(mut audio_data_buffer) = audio_data_clone.lock() {
                            for &sample in data {
                                audio_data_buffer.push(sample);
                            }
                        }
                    }
                },
                err_fn,
                None,
            )?
        }
        _ => todo!(),
    };

    stream.play()?;
    println!("Recording audio input. Press Ctrl+C to stop...");

    // Keep running until Ctrl+C is received
    while running.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    println!("Stopping audio input...");

    let audio_data_clone = audio_data.lock().unwrap().clone();

    // Calculate and print original audio duration
    let original_duration_sec =
        audio_data_clone.len() as f32 / (sample_rate as f32 * channels as f32);
    println!(
        "Original audio duration: {:.2} seconds",
        original_duration_sec
    );

    // Resample
    let resampled_audio = resample_to_16khz(&audio_data_clone, sample_rate, channels)?;
    let resampled_duration_sec = resampled_audio.len() as f32 / (16000.0 * channels as f32);
    println!(
        "Resampled audio duration: {:.2} seconds",
        resampled_duration_sec
    );

    // Only transcribe if we have at least 1 second of audio
    if resampled_duration_sec >= 1.0 {
        let transcription = transcribe(&resampled_audio);
        println!("{}", "Transcription:".bright_green().bold());
        println!("{}", transcription.bright_white());
    } else {
        println!("Audio too short for transcription (need at least 1 second)");
    }
    Ok(())
}
