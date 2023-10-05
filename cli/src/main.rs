//! Records a WAV file (roughly 3 seconds long) using the default input device and config.
//!
//! The input data is recorded to "$CARGO_MANIFEST_DIR/recorded.wav".

use anyhow::Context;
use clap::Parser;
use gwhisper::recogntion::RecognitionOptions;
use gwhisper::recording::Recorder;
use gwhisper::{recogntion::Recognition, recording};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let opt = Opt::parse();
    let recognition = Recognition::new(&opt.model)?;

    let (device, config) = recording::whisper_config(&opt.device)?;

    println!("Default input config: {:?}", config);

    // A flag to indicate that recording is in progress.
    println!("Begin recording...");

    let mut recorder = Recorder::new(device, config.into());
    recorder.start().context("recording")?;

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

    let options = RecognitionOptions {
        lang: opt.lang,
        ..Default::default()
    };
    let output = recognition
        .recognize(&audio, options)
        .expect("whisper error");

    println!("{}", output.trim());

    Ok(())
}
