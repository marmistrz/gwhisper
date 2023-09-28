//! Records a WAV file (roughly 3 seconds long) using the default input device and config.
//!
//! The input data is recorded to "$CARGO_MANIFEST_DIR/recorded.wav".

use clap::Parser;
use derivative::Derivative;
use gtk::prelude::*;
use gtk::traits::ButtonExt;
use gwhisper::recogntion::all_langs;
use gwhisper::recording::Recorder;
use relm4::gtk::gdk;
use relm4::{component, prelude::*};
use relm4_components::open_dialog::{
    OpenDialog, OpenDialogMsg, OpenDialogResponse, OpenDialogSettings,
};
use relm4_components::simple_combo_box::SimpleComboBox;
use std::path::{Path, PathBuf};

mod worker;

use worker::{RecognitionMsg, RecognitionWorker};

const APP_NAME: &str = "gwhisper";

#[derive(Parser, Debug)]
#[command(version, about = APP_NAME, long_about = None)]
struct Opt {
    /// The audio device to use
    #[arg(short, long, default_value_t = String::from("default"))]
    device: String,
}

fn main() {
    // TODO default to log level info
    env_logger::init();
    let app = RelmApp::new("relm4.test.simple_manual");
    app.run::<App>(());
}

#[derive(Debug)]
enum Msg {
    ToggleRecord,
    ChooseModel,
    LoadModel(PathBuf),
    WriteText(String),
    CopyText,
    SetLang(String),
    Ignore,
}

#[derive(Derivative)]
#[derivative(Default)]
pub struct AppState {
    buffer: gtk::TextBuffer,
    working: bool,
    recording: bool,
    #[derivative(Default(value = r#""not loaded".into()"#))]
    model_path: String,
}

struct App {
    recorder: Recorder,
    recognition_worker: Controller<RecognitionWorker>,

    app_state: AppState,

    open_dialog: Controller<OpenDialog>,
    lang_combo: Controller<SimpleComboBox<&'static str>>,
}

#[component]
impl SimpleComponent for App {
    type Input = Msg;
    type Output = ();
    type Init = ();

    view! {
        gtk::ApplicationWindow {
            set_title: Some("Simple app"),
            set_default_size: (800, 600),

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 5,
                    set_margin_all: 5,
                    set_hexpand: true,

                    gtk::TextView {
                        #[watch]
                        set_buffer: Some(&model.app_state.buffer),
                        set_vexpand: true,
                    },
                    gtk::Button {
                        #[watch]
                        set_label: model.recording_label(), // FIXME threads
                        #[watch]
                        set_sensitive: !model.app_state.working, // FIXME && model.resources.whisper_ready(),
                        connect_clicked[sender] => move |_| {
                            sender.input(Msg::ToggleRecord);
                        }
                    },
                    gtk::Label {
                        #[watch]
                        set_label: &format!("Model path: {}", model.app_state.model_path)
                    }
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    #[local_ref]
                    lang_combo -> gtk::ComboBoxText {},
                    gtk::Button {
                        set_label: "Load model",
                        connect_clicked[sender] => move |_| {
                            sender.input(Msg::ChooseModel);
                        }
                    },
                    gtk::Button {
                        set_label: "Copy text",
                        connect_clicked[sender] => move |_| {
                            sender.input(Msg::CopyText);
                        }
                    }
                }
            }

        }
    }

    // Initialize the UI.
    fn init(
        _: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let recorder = Recorder::default();
        let app_state = AppState::default();

        let open_dialog = OpenDialog::builder()
            .transient_for_native(root)
            .launch(OpenDialogSettings::default())
            .forward(sender.input_sender(), |response| match response {
                OpenDialogResponse::Accept(path) => Msg::LoadModel(path),
                OpenDialogResponse::Cancel => Msg::Ignore,
            });

        // TODO: add support for lang = "auto"
        let lang_combo = SimpleComboBox {
            variants: all_langs().collect(),
            active_index: Some(0),
        };
        let lang_combo = SimpleComboBox::builder()
            .launch(lang_combo)
            .forward(sender.input_sender(), |_| Msg::SetLang("auto".into())); // FIXME set the lang

        let recognition_worker = RecognitionWorker::builder().launch(None).forward(
            sender.input_sender(),
            |msg| match msg {
                Ok(text) => Msg::WriteText(text),
                Err(e) => {
                    println!("TODO: show an error dialog: {}", e);
                    Msg::Ignore
                }
            },
        );

        let model = Self {
            recorder,
            recognition_worker,
            app_state,
            open_dialog,
            lang_combo,
        };

        let lang_combo = model.lang_combo.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            Msg::ToggleRecord => self.toggle_record(),
            Msg::ChooseModel => self.open_dialog.emit(OpenDialogMsg::Open),
            Msg::CopyText => self.copy_text(),
            Msg::WriteText(text) => self.write_text(&text),
            Msg::SetLang(_) => self.set_lang(),
            Msg::LoadModel(path) => self.load_model(&path),
            Msg::Ignore => {}
        }
    }
}

impl App {
    fn toggle_record(&mut self) {
        if self.recorder.is_stopped() {
            self.app_state.recording = true;
            self.recorder
                .start()
                .expect("TODO: show a dialog why recording failed to start");
        } else {
            self.app_state.recording = false;
            self.app_state.working = true;
            let audio = self.recorder.stop();
            self.recognition_worker
                .emit(RecognitionMsg::Transcribe(audio));
        }
    }

    fn load_model(&mut self, path: &Path) {
        let path = path.to_str().expect("invalid utf8").to_owned();
        self.recognition_worker
            .emit(RecognitionMsg::LoadModel(path.clone()));
        // TODO: only do this if loading succeededs
        self.app_state.model_path = path;
    }

    fn copy_text(&self) {
        let clipboard = gdk::Display::default().unwrap().clipboard();
        let buffer = &self.app_state.buffer;
        let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true);
        clipboard.set_text(text.as_str());
    }

    fn set_lang(&self) {
        let lang = self
            .lang_combo
            .model()
            .get_active_elem()
            .expect("no active element")
            .to_string();
        self.recognition_worker.emit(RecognitionMsg::SetLang(lang))
    }

    fn write_text(&mut self, text: &str) {
        self.app_state.buffer.set_text(text);
        self.app_state.working = false;
    }

    fn recording_label(&self) -> &str {
        if self.app_state.recording {
            "Recording..."
        } else {
            "Record"
        }
    }
}
