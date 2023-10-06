use gwhisper::recogntion::{Recognition, RecognitionOptions, WhisperError};
use relm4::{ComponentSender, Worker};

pub(crate) struct RecognitionWorker {
    recognition: Option<Recognition>,
    lang: String, // TODO: use a Language type with a sane default
}

#[derive(Debug)]
pub(crate) enum RecognitionMsg {
    LoadModel(String),
    Transcribe(Vec<f32>),
    SetLang(String),
}

impl Worker for RecognitionWorker {
    type Init = Option<Recognition>; // model: TODO: pass a PathBuf, initialization might take a while
    type Input = RecognitionMsg; // recording: TODO perhaps send it as a reference??
    type Output = Result<String, WhisperError>;

    fn init(recognition: Self::Init, _sender: ComponentSender<Self>) -> Self {
        Self {
            recognition,
            lang: "auto".into(),
        }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            RecognitionMsg::Transcribe(audio) => {
                let options = RecognitionOptions {
                    lang: self.lang.clone(),
                    ..Default::default()
                };
                let rec_result = self
                    .recognition
                    .as_ref()
                    .expect("Recognition not yet initialized")
                    .recognize(&audio, options);
                sender.output(rec_result).expect("channel closed");
            }
            RecognitionMsg::LoadModel(model) => {
                match Recognition::new(&model) {
                    Ok(rec) => self.recognition = Some(rec),
                    Err(e) => sender.output(Err(e)).expect("channel closed"),
                };
            }
            RecognitionMsg::SetLang(lang) => self.lang = lang,
        }
    }
}
