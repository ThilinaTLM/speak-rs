use anyhow::Result;
use rubato::{Resampler, SincFixedIn, SincInterpolationType, WindowFunction};

pub fn resample_to_16khz(
    audio_data: &[f32],
    sample_rate: u32,
    channels: usize,
) -> Result<Vec<f32>> {
    if sample_rate == 16000 {
        return Ok(audio_data.to_vec());
    }

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

    Ok(output)
}
