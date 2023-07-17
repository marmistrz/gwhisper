//! Records a WAV file (roughly 3 seconds long) using the default input device and config.
//!
//! The input data is recorded to "$CARGO_MANIFEST_DIR/recorded.wav".

use anyhow::Context;
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{SampleFormat, SampleRate, SupportedStreamConfig};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod recording;
mod recogntion;

use recording::Recorder;

use crate::recogntion::recognize;

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
    #[arg(short, long, default_value_t = String::from("auto"))]
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

    let recorder = Recorder::default();
    let recorder = recorder
        .start(&device, &config.into())
        .context("recording")?;

    // Let recording go for roughly three seconds.
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    println!("Waiting for Ctrl-C...");
    while running.load(Ordering::SeqCst) {}

    let audio = recorder.stop();

    println!("Recording complete, len = {}!", audio.len());

    let output = recognize(&audio, &opt.model, &opt.lang);

    println!("{}", output.trim());

    Ok(())
}
