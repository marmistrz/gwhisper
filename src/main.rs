//! Records a WAV file (roughly 3 seconds long) using the default input device and config.
//!
//! The input data is recorded to "$CARGO_MANIFEST_DIR/recorded.wav".

use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, SupportedStreamConfig};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use whisper_rs::{FullParams, WhisperContext};

fn default_model() -> String {
    String::from("/usr/share/whisper.cpp-model-base.en/base.en.bin")
}

#[derive(Parser, Debug)]
#[command(version, about = "CPAL record_wav example", long_about = None)]
struct Opt {
    /// The audio device to use
    #[arg(short, long, default_value_t = String::from("default"))]
    device: String,
    /// The file containing the model
    #[arg(short, long, default_value_t = default_model())]
    model: String,
    /// The language for transcription. Use `auto` for auto-detection.
    #[arg(short, long, default_value_t = default_model())]
    lang: String,
}

const CHANNELS: u16 = 1;
const SAMPLE_RATE: SampleRate = SampleRate(16000);
const SAMPLE_FORMAT: SampleFormat = SampleFormat::F32;

fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();
    let host = cpal::default_host();

    // Set up the input device and stream with the default input config.
    let device = if opt.device == "default" {
        host.default_input_device()
    } else {
        host.input_devices()?
            .find(|x| x.name().map(|y| y == opt.device).unwrap_or(false))
    }
    .expect("failed to find input device");

    println!("Input device: {}", device.name()?);

    let config = device
        .default_input_config()
        .expect("Failed to get default input config");
    let config = SupportedStreamConfig::new(
        CHANNELS,
        SAMPLE_RATE,
        config.buffer_size().clone(),
        SAMPLE_FORMAT,
    );

    println!("Default input config: {:?}", config);

    // A flag to indicate that recording is in progress.
    println!("Begin recording...");

    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };

    let audio: Vec<f32> = Vec::new();
    let audio = Arc::new(Mutex::new(audio));

    let stream = {
        let audio = audio.clone();
        device.build_input_stream(
            &config.into(),
            move |data, _: &_| audio.lock().unwrap().extend(data),
            err_fn,
            None,
        )?
    };

    stream.play()?;

    // Let recording go for roughly three seconds.
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    println!("Waiting for Ctrl-C...");
    while running.load(Ordering::SeqCst) {}

    drop(stream);
    println!("Recording complete, len = {}!", audio.lock().unwrap().len());

    let ctx = WhisperContext::new(&opt.model).expect("Failed to create WhisperContext");
    let mut params = FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some(&opt.lang));

    let mut state = ctx.create_state().expect("test");
    state
        .full(params, audio.lock().unwrap().as_ref())
        .expect("full failed");

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

    println!("{}", output.trim());

    Ok(())
}
