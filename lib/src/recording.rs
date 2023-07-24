pub use cpal;
use cpal::{
    traits::{DeviceTrait, StreamTrait, HostTrait},
    Stream, Device, StreamConfig, SupportedStreamConfig, SampleRate, SampleFormat,
};
use std::sync::{Arc, Mutex};

type Audio = Vec<f32>;
type AudioContainer = Arc<Mutex<Audio>>;

pub struct StoppedRecorder {
    audio: AudioContainer,
    device: cpal::Device,
    config: cpal::StreamConfig,
}

impl StoppedRecorder {
    pub fn new(device: cpal::Device, config: cpal::StreamConfig) -> Self {
        Self {
            audio: Default::default(),
            device,
            config,
        }
    }

    pub fn start(self) -> anyhow::Result<StartedRecorder> {
        // TODO use a proper error type and remove anyhow
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

        Ok(StartedRecorder {
            audio: self.audio,
            stream,
        })
    }
}

pub struct StartedRecorder {
    audio: AudioContainer,
    stream: Stream,
}

impl StartedRecorder {
    pub fn stop(self) -> Audio {
        drop(self.stream);
        self.audio.lock().unwrap().to_vec() // TODO use option and move instead of copying
    }
}

pub enum Recorder {
    Stopped(StoppedRecorder),
    Started(StartedRecorder),
}

impl Recorder {
    pub fn new(device: cpal::Device, config: cpal::StreamConfig) -> Self {
        Self::Stopped(StoppedRecorder::new(device, config))
    }

    pub fn start(self) -> anyhow::Result<Recorder> {
        if let Self::Stopped(recorder) = self {
            Ok(Self::Started(recorder.start()?))
        } else {
            panic!("Recorder should be stopped")
        }
    }

    pub fn stop(self) -> anyhow::Result<(Recorder, Audio)> {
        if let Self::Started(recorder) = self {
            Ok(Self::Stopped(recorder.stop()?))
        } else {
            panic!("Recorder should be stopped")
        }
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
