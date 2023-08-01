pub use cpal;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, SampleFormat, SampleRate, Stream, SupportedStreamConfig, StreamConfig,
};
use std::sync::{Arc, Mutex};

type Audio = Vec<f32>;
type AudioContainer = Arc<Mutex<Audio>>;

pub struct Recorder {
    audio: AudioContainer,
    device: Device,
    config: StreamConfig,
    stream: Option<Stream>,
}

impl Recorder {
    pub fn new(device: cpal::Device, config: cpal::StreamConfig) -> Self {
        Self {
            audio: Default::default(),
            device,
            config,
            stream: None,
        }
    }

    pub fn is_stopped(&self) -> bool {
        self.stream.is_none()
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        assert!(self.stream.is_none());
        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };

        let stream = {
            let audio = self.audio.clone();
            self.device.build_input_stream(
                &self.config,
                move |data, _: &_| audio.lock().unwrap().extend(data),
                err_fn,
                None,
            )?
        };

        stream.play()?;
        self.stream = Some(stream);

        Ok(())
    }

    pub fn stop(&mut self) -> Audio {
        let stream = self.stream.take().expect("Recorder should be started");
        drop(stream);
        let audio = self.audio.lock().unwrap().clone(); // FIXME move
        audio
    }
}

// whisper-specfic stuff below
const CHANNELS: u16 = 1;
const SAMPLE_RATE: SampleRate = SampleRate(16000);
const SAMPLE_FORMAT: SampleFormat = SampleFormat::F32;

pub fn whisper_config(device: &str) -> anyhow::Result<(Device, SupportedStreamConfig)> {
    let host = cpal::default_host();

    // Set up the input device and stream with the default input config.
    let device = if device == "default" {
        host.default_input_device()
    } else {
        host.input_devices()?
            .find(|x| x.name().map(|y| y == device).unwrap_or(false))
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

    Ok((device, config))
}
