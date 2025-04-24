#![allow(dead_code)]

use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct TranscribeOutput {
    pub combined: String,
    pub segments: Vec<Segment>,
}

pub struct Segment {
    pub start: f32,
    pub end: f32,
    pub text: String,
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

    pub fn transcribe(&self, audio_data: &[f32]) -> TranscribeOutput {
        let mono_audio = whisper_rs::convert_stereo_to_mono_audio(audio_data)
            .expect("failed to convert audio to mono");

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(true);

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

            // Add formatted segment to combined transcription
            combined.push_str(&format!("[{} - {}]: {}\n", start, end, text.clone()));

            // Add segment to the segments list
            segments.push(Segment {
                start: start as f32,
                end: end as f32,
                text: text,
            });
        }

        TranscribeOutput { combined, segments }
    }
}
