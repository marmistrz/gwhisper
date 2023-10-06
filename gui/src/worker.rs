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
    type Init = Option<String>;
    type Input = RecognitionMsg;
    type Output = Result<String, WhisperError>;

    fn init(model_path: Self::Init, _sender: ComponentSender<Self>) -> Self {
        // TODO: propagate this error to the GUI
        let recognition =
            model_path.map(|path| Recognition::new(&path).expect("Error creating recognition"));
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
