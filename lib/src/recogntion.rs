pub use whisper_rs::WhisperError;
use whisper_rs::{get_lang_str, FullParams, WhisperContext};

#[derive(Debug)]
pub struct Recognition {
    ctx: WhisperContext,
}

#[derive(Default)]
pub struct RecognitionOptions {
    pub lang: String, // TODO: use a custom type with sane defaults
    pub progress_closure: Option<Box<dyn FnMut(i32)>>,
}

impl Recognition {
    pub fn new(model: &str) -> Result<Self, WhisperError> {
        println!("whisper system info: {}", whisper_rs::print_system_info());
        let ctx = WhisperContext::new(model)?;
        Ok(Self { ctx })
    }

    pub fn recognize(
        &self,
        audio: &[f32],
        options: RecognitionOptions,
    ) -> Result<String, WhisperError> {
        let mut params = FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some(options.lang.as_ref()));
        params.set_no_context(true);
        if let Some(closure) = options.progress_closure {
            params.set_progress_callback_safe(closure);
        }

        let mut state = self.ctx.create_state()?;
        state.full(params, audio)?;

        let mut output = String::new();
        let num_segments = state.full_n_segments()?;
        println!("num segments: {}", num_segments);
        for i in 0..num_segments {
            let segment = state.full_get_segment_text(i)?;
            output.push_str(&segment)
        }

        Ok(output)
    }
}

pub fn all_langs() -> impl Iterator<Item = &'static str> {
    let num_langs = whisper_rs::get_lang_max_id();
    (0..num_langs)
        .map(|id| get_lang_str(id).unwrap_or_else(|| panic!("No lang name for id = {}", id)))
}
