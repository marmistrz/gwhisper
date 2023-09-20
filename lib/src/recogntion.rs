pub use whisper_rs::WhisperError;
use whisper_rs::{get_lang_str, FullParams, WhisperContext};

#[derive(Debug)]
pub struct Recognition {
    ctx: WhisperContext,
    lang: String,
}

const DEFAULT_LANG: &str = "auto";

impl Recognition {
    pub fn new(model: &str) -> Result<Self, WhisperError> {
        println!("whisper system info: {}", whisper_rs::print_system_info());
        let ctx = WhisperContext::new(model)?;
        Ok(Self {
            ctx,
            lang: DEFAULT_LANG.to_owned(),
        })
    }

    pub fn set_lang(&mut self, lang: &str) {
        self.lang = lang.to_owned();
    }

    pub fn set_lang_id(&mut self, lang_id: i32) {
        let lang = get_lang_str(lang_id).expect("unknown lang id");
        self.set_lang(lang);
    }

    pub fn recognize(&self, audio: &[f32]) -> Result<String, WhisperError> {
        let mut params = FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some(&self.lang));
        params.set_no_context(true);

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
