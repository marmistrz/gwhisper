use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, SampleRate, SupportedStreamConfig};
use std::sync::{Arc, Mutex};
use whisper_rs::{FullParams, WhisperContext};

const CHANNELS: u16 = 1;
const SAMPLE_RATE: SampleRate = SampleRate(16000);
const SAMPLE_FORMAT: SampleFormat = SampleFormat::F32;

struct SpeechRecognition {
    device: Device,
    config: SupportedStreamConfig,
    stream: Option<cpal::Stream>,
}

impl SpeechRecognition {
    pub fn new(device: &str) -> anyhow::Result<Self> {
        let host = cpal::default_host();

        // Set up the input device and stream with the default input config.
        let device = if device == "default" {
            host.default_input_device()
        } else {
            host.input_devices()?
                .find(|x| x.name().map(|y| y == device).unwrap_or(false))
        }
        .expect("failed deviceto find input device");

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

        Ok(Self {
            device,
            config,
            stream: None,
        })
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        println!("Begin recording...");

        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };

        let audio: Vec<f32> = Vec::new();
        let audio = Arc::new(Mutex::new(audio));

        let stream = {
            let audio = audio.clone();
            self.device.build_input_stream(
                &self.config.into(),
                move |data, _: &_| {
                    println!("Got {} bytes", data.len());
                    audio.lock().unwrap().extend(data)
                },
                err_fn,
                None,
            )?
        };

        stream.play()?;
        Ok(())
    }

    pub fn stop(&self) -> String {
        Default::default() // FIXME
    }
}

fn voice_recognition() -> anyhow::Result<String> {
    // A flag to indicate that recording is in progress.

    // Let recording go for roughly three seconds.
    std::thread::sleep(std::time::Duration::from_secs(3));
    drop(stream);
    println!("Recording complete, len = {}!", audio.lock().unwrap().len());

    let mut ctx = WhisperContext::new("/usr/share/whisper.cpp-model-base.en/base.en.bin")
        .expect("Failed to create WhisperContext");
    let params = FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 10 });
    ctx.full(params, audio.lock().unwrap().as_ref())
        .expect("full failed");

    let num_segments = ctx.full_n_segments();
    let segments: Vec<_> = (0..num_segments)
        .map(|i| ctx.full_get_segment_text(i).expect("failed to get segment"))
        .collect();
    let recognized = segments.join(" ");
    Ok(recognized)
}
