pub use cpal;
use cpal::{
    traits::{DeviceTrait, StreamTrait},
    Stream,
};
use std::sync::{Arc, Mutex};

type Audio = Vec<f32>;
type AudioContainer = Arc<Mutex<Audio>>;

#[derive(Default)]
pub struct Recorder {
    audio: AudioContainer,
}

impl Recorder {
    pub fn start(
        self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
    ) -> anyhow::Result<StartedRecorder> { // TODO use a proper error type and remove anyhow
        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };

        let stream = {
            let audio = self.audio.clone();
            device.build_input_stream(
                config,
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
