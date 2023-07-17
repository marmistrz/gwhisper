use whisper_rs::{FullParams, WhisperContext, WhisperError};

pub struct Recognition {
    ctx: WhisperContext,
}

impl Recognition {
    pub fn new(model: &str) -> Result<Self, WhisperError> {
        let ctx = WhisperContext::new(model)?;
        Ok(Self { ctx })
    }

    pub fn recognize(&self, audio: &[f32], lang: &str) -> String {
        let mut params = FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some(lang));

        let mut state = self.ctx.create_state().expect("test");
        state.full(params, &audio).expect("full failed");

        let mut output = String::new();
        let num_segments = state.full_n_segments().expect("FIXME");
        println!("num segments: {}", num_segments);
        for i in 0..num_segments {
            let segment = state
                .full_get_segment_text(i)
                .expect("failed to get segment");
            // let start_timestamp = ctx.full_get_segment_t0(i);
            // let end_timestamp = ctx.full_get_segment_t1(i);
            // println!("[{} - {}]: {}", start_timestamp, end_timestamp, segment);
            output.push_str(&segment)
        }

        output
    }
}
