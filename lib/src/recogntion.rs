use whisper_rs::{FullParams, WhisperContext, WhisperError};

pub struct Recognition {
    ctx: WhisperContext,
    lang: String,
}

const DEFAULT_LANG: &str = "auto";

impl Recognition {
    pub fn new(model: &str) -> Result<Self, WhisperError> {
        let ctx = WhisperContext::new(model)?;
        Ok(Self {
            ctx,
            lang: DEFAULT_LANG.to_owned(),
        })
    }

    pub fn set_lang(&mut self, lang: &str) {
        self.lang = lang.to_owned();
    }

    pub fn recognize(&self, audio: &[f32]) -> String {
        let mut params = FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some(&self.lang));

        let mut state = self.ctx.create_state().expect("test");
        state.full(params, &audio).expect("full failed");

        let mut output = String::new();
        let num_segments = state.full_n_segments().expect("FIXME");
        println!("num segments: {}", num_segments);
        for i in 0..num_segments {
            let segment = state
                .full_get_segment_text(i)
                .expect("failed to get segment");
            output.push_str(&segment)
        }

        output
    }
}
