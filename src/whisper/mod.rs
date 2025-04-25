#![allow(dead_code)]

use anyhow::Result;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

mod resample;

pub struct InputAudio<'a> {
    pub data: &'a [f32],
    pub sample_rate: u32,
    pub channels: usize,
}

pub struct TranscribeOutput {
    pub combined: String,
    pub segments: Vec<Segment>,
}

pub struct Segment {
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub confidence: f32,
}

impl PartialEq for Segment {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end && self.text == other.text
    }
}

pub struct SimpleTranscriber {
    ctx: WhisperContext,
}

impl SimpleTranscriber {
    pub fn new() -> Self {
        let path_to_model = "models/ggml-small.en.bin";
        let mut ctx_params = WhisperContextParameters::default();
        ctx_params.use_gpu(true);
        let ctx = WhisperContext::new_with_params(&path_to_model, ctx_params)
            .expect("failed to load model");

        Self { ctx }
    }

    pub fn transcribe(&self, audio_data: &InputAudio) -> Result<TranscribeOutput> {
        let resampled_audio = match resample::resample_to_16khz(
            audio_data.data,
            audio_data.sample_rate,
            audio_data.channels,
        ) {
            Ok(audio) => audio,
            Err(_) => return Err(anyhow::anyhow!("failed to resample audio")),
        };
        if resampled_audio.len() < 16000 {
            return Err(anyhow::anyhow!("resampled audio is too short"));
        }

        let mono_audio = whisper_rs::convert_stereo_to_mono_audio(&resampled_audio)
            .expect("failed to convert audio to mono");

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(true);
        params.set_audio_ctx(768);
        params.set_no_speech_thold(0.5);
        params.set_n_threads(2);

        // Run the model
        let mut state = self.ctx.create_state().expect("failed to create state");
        state
            .full(params, &mono_audio[..])
            .expect("failed to run model");

        // Fetch the results
        let num_segments = state
            .full_n_segments()
            .expect("failed to get number of segments");

        let mut combined = String::new();
        let mut segments = Vec::new();

        for i in 0..num_segments {
            let text = state
                .full_get_segment_text(i)
                .expect("failed to get segment");
            let start = state
                .full_get_segment_t0(i)
                .expect("failed to get segment start timestamp");
            let end = state
                .full_get_segment_t1(i)
                .expect("failed to get segment end timestamp");

            let n_tok = state.full_n_tokens(i)?;
            let mut sum_logprob = 0.0_f32;

            for t in 0..n_tok {
                let tok = state.full_get_token_data(i, t)?; // tok.plog is log-p
                sum_logprob += tok.plog;
            }

            let avg_logprob = sum_logprob / n_tok as f32;
            let confidence = avg_logprob.exp();

            // Add formatted segment to combined transcription
            combined.push_str(&text);
            segments.push(Segment {
                start: start as usize,
                end: end as usize,
                text: text,
                confidence: confidence,
            });
        }

        Ok(TranscribeOutput { combined, segments })
    }
}
