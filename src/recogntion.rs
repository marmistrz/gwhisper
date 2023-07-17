
use whisper_rs::{FullParams, WhisperContext};

pub fn recognize(audio: &[f32], model: &str, lang: &str) -> String {
    let ctx = WhisperContext::new(model).expect("Failed to create WhisperContext");
    let mut params = FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some(lang));

    let mut state = ctx.create_state().expect("test");
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