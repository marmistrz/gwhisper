//! Records a WAV file (roughly 3 seconds long) using the default input device and config.
//!
//! The input data is recorded to "$CARGO_MANIFEST_DIR/recorded.wav".

use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, SupportedStreamConfig};
use whisper_rs::{WhisperContext, FullParams};
use std::sync::{Arc, Mutex};

#[derive(Parser, Debug)]
#[command(version, about = "CPAL record_wav example", long_about = None)]
struct Opt {
    /// The audio device to use
    #[arg(short, long, default_value_t = String::from("default"))]
    device: String,
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
            move |data, _: &_| {
                println!("Got {} bytes", data.len());
                audio.lock().unwrap().extend(data)
            },
            err_fn,
            None,
        )?
    };

    stream.play()?;

    // Let recording go for roughly three seconds.
    std::thread::sleep(std::time::Duration::from_secs(3));
    drop(stream);
    println!("Recording complete, len = {}!", audio.lock().unwrap().len());

    let mut ctx = WhisperContext::new("/usr/share/whisper.cpp-model-base.en/base.en.bin").expect("Failed to create WhisperContext");
    let params = FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 10 });
    ctx.full(params, audio.lock().unwrap().as_ref()).expect("full failed");

    let num_segments = ctx.full_n_segments();
    for i in 0..num_segments {
        let segment = ctx.full_get_segment_text(i).expect("failed to get segment");
        let start_timestamp = ctx.full_get_segment_t0(i);
        let end_timestamp = ctx.full_get_segment_t1(i);
        println!("[{} - {}]: {}", start_timestamp, end_timestamp, segment);
    }

    Ok(())
}


