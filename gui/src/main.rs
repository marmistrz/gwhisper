//! Records a WAV file (roughly 3 seconds long) using the default input device and config.
//!
//! The input data is recorded to "$CARGO_MANIFEST_DIR/recorded.wav".

use clap::Parser;
use derivative::Derivative;
use gtk::prelude::*;
use gtk::traits::ButtonExt;
use gwhisper::recogntion::{all_langs, Recognition};
use gwhisper::recording::{self, Recorder};
use relm4::gtk::gdk;
use relm4::{component, prelude::*};
use relm4_components::open_dialog::{
    OpenDialog, OpenDialogMsg, OpenDialogResponse, OpenDialogSettings,
};
use relm4_components::simple_combo_box::SimpleComboBox;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

const APP_NAME: &str = "gwhisper";

#[derive(Parser, Debug)]
#[command(version, about = APP_NAME, long_about = None)]
struct Opt {
    /// The audio device to use
    #[arg(short, long, default_value_t = String::from("default"))]
    device: String,
}

fn main() {
    let app = RelmApp::new("relm4.test.simple_manual");
    app.run::<App>(());
    // Set up the input device and stream with the default input config.
    // let (device, config) = recording::whisper_config("default").expect("FIXME");

    // let recorder = Recorder::new(device, config.into());
    // let app = Application {
    //     recorder: Rc::new(Mutex::new(recorder)),
    //     recognition: Arc::new(Mutex::new(None)),
    // };

    // if gtk::init().is_err() {
    //     panic!("Failed to initialize GTK.");
    // }
    // app.setup();
    // gtk::main() FIXME
}

#[derive(Debug)]
enum Msg {
    ToggleRecord,
    ChooseModel,
    LoadModel(PathBuf),
    CopyText,
    SetLang(usize),
    RecvText(String),
    Ignore,
    // Quit,
}

struct Resources {
    recorder: Rc<Mutex<Recorder>>,
    recognition: Arc<Mutex<Option<Recognition>>>,
}

impl Default for Resources {
    fn default() -> Self {
        // Set up the input device and stream with the default input config.
        let (device, config) = recording::whisper_config("default").expect("FIXME");

        let recorder = Recorder::new(device, config.into());
        Self {
            recorder: Rc::new(Mutex::new(recorder)),
            recognition: Arc::new(Mutex::new(None)),
        }
    }
}

impl Resources {
    fn whisper_ready(&self) -> bool {
        self.recognition.lock().unwrap().is_some()
    }
}

#[derive(Derivative)]
#[derivative(Default)]
pub struct AppState {
    buffer: gtk::TextBuffer,
    working: bool,
    recording: bool,
    model_path: String,
    #[derivative(Default(value = "\"auto\".into()"))]
    lang: String,
}

struct App {
    resources: Resources,
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
                        set_sensitive: !model.app_state.working && model.resources.whisper_ready(),
                        connect_clicked[sender] => move |_| {
                            sender.input(Msg::ToggleRecord);
                        }
                    },
                    gtk::Label {
                        #[watch]
                        set_label: &model.app_state.model_path
                    }
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    gtk::ComboBoxText {},
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
        let resources = Resources::default();
        let app_state = AppState::default();

        let open_dialog = OpenDialog::builder()
            .transient_for_native(root)
            .launch(OpenDialogSettings::default())
            .forward(sender.input_sender(), |response| match response {
                OpenDialogResponse::Accept(path) => Msg::LoadModel(path),
                OpenDialogResponse::Cancel => Msg::Ignore,
            });

        let lang_combo = SimpleComboBox {
            variants: all_langs().collect(),
            active_index: Some(0),
        };
        let lang_combo = SimpleComboBox::builder().launch(lang_combo).forward(
            sender.input_sender(),
            |id| Msg::SetLang(id), // FIXME
        );

        let model = Self {
            resources,
            app_state,
            open_dialog,
            lang_combo,
        };

        let widgets = view_output!();

        // for lang in all_langs() {
        //     widgets.lang_combo_box.append_text(lang);
        //     // TODO set default as active
        // }
        //     ui.lang_combo_box.connect_changed({
        //         let recognition = self.recognition.clone();
        //         move |combo| {
        //             let lang = combo.active_text().expect("should be selected");
        //             recognition
        //                 .lock()
        //                 .unwrap()
        //                 .as_mut()
        //                 .expect("model not initialized")
        //                 .set_lang(lang.as_str());
        //         }
        //     });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            Msg::ToggleRecord => self.toggle_record(),
            Msg::ChooseModel => self.open_dialog.emit(OpenDialogMsg::Open),
            Msg::CopyText => self.copy_text(),
            Msg::SetLang(lang_id) => self.set_lang(lang_id),
            Msg::RecvText(_) => todo!(),
            Msg::LoadModel(path) => self.load_model(&path),
            Msg::Ignore => {}
        }
    }
}

impl App {
    fn toggle_record(&mut self) {
        let recorder = self.resources.recorder.clone();
        let recognition = self.resources.recognition.clone();
        let mut recorder = recorder.lock().unwrap();
        if recorder.is_stopped() {
            recorder.start().expect("FIXME");
        } else {
            self.app_state.working = true;
            let audio = recorder.stop();

            // FIXME use a worker/thread
            //thread::spawn({
            //let recognition = recognition.clone();
            //let tx = data_tx.clone();
            //move || {
            let text = recognition
                .lock()
                .unwrap()
                .as_ref()
                .expect("record button should be insensitive")
                .recognize(&audio)
                .expect("TODO: show a dialog for whisper error");
            self.app_state.buffer.set_text(&text);
            //tx.send(text).expect("channel error");
            // }
        }
    }

    fn load_model(&mut self, path: &Path) {
        let path = path.to_str().expect("invalid utf8");

        match Recognition::new(path) {
            Ok(rec) => {
                *self.resources.recognition.lock().unwrap() = Some(rec);
                self.app_state.model_path = format!("Model: {}", path);
            }
            Err(e) => {
                // let dialog = gtk::MessageDialog::builder()
                //     .parent(&ui.window)
                //     .message_type(gtk::MessageType::Error)
                //     .text(e.to_string())
                //     .buttons(gtk::ButtonsType::Ok)
                //     .build();
                // dialog.run();
                // dialog.close();
            }
        }
    }

    fn copy_text(&self) {
        let clipboard = gdk::Display::default().unwrap().clipboard();
        let ref buffer = self.app_state.buffer;
        let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true);
        clipboard.set_text(text.as_str());
    }

    fn set_lang(&self, lang_id: usize) {
        if let Some(ref mut rec) = *self.resources.recognition.lock().unwrap() {
            let lang_id = lang_id.try_into().unwrap();
            rec.set_lang_id(lang_id)
        }
    }

    fn recording_label(&self) -> &str {
        if self.app_state.recording {
            "Recording..."
        } else {
            "Record"
        }
    }
}

// fn setup(&self) {
//     let ui = Rc::new(Ui::default());
//     let (data_tx, data_rx) = glib::MainContext::channel(glib::Priority::DEFAULT);

//     data_rx.attach(None, {
//         let ui = ui.clone();
//         move |text: String| {
//             ui.record_button.set_sensitive(true);
//             let buffer = ui.text_view.buffer().expect("buffer");
//             let mut end = buffer.end_iter();
//             buffer.insert(&mut end, &text);
//             ui.record_button.set_label("Record");

//             glib::ControlFlow::Continue
//         }
//     });

//     let clipboard = Clipboard::get(&gdk::SELECTION_CLIPBOARD);
//     ui.copy_button.connect_clicked({
//         let ui = ui.clone();
//         move |_| {
//             let buffer = ui.text_view.buffer().expect("textview buffer");
//             let text = buffer
//                 .text(&buffer.start_iter(), &buffer.end_iter(), true)
//                 .expect("buffer text");
//             clipboard.set_text(text.as_str());
//         }
//     });

//     ui.model_choice_button.connect_clicked({
//         let recognition = self.recognition.clone();
//         let ui = ui.clone();
//         move |_| {
//             let dialog = FileChooserDialog::new(
//                 Some("Open model"),
//                 Some(&ui.window),
//                 gtk::FileChooserAction::Open,
//             );
//             dialog.add_button("OK", ResponseType::Accept);
//             dialog.add_button("Cancel", ResponseType::Cancel);

//             let resp = dialog.run();
//             dialog.close(); // FIXME: the dialog is not really closed in case of an error. Perhaps idle_add or sth?
//             if let ResponseType::Accept = resp {
//                 let model = dialog
//                     .filename()
//                     .expect("TODO: when can the filename be none?");
//                 let mut guard = recognition.lock().unwrap();
//                 Self::set_model(&mut *guard, ui.as_ref(), &model);
//             }
//         }
//     });

//     // Present window
//     ui.window.show_all();
//     // TODO the executable should terminate when the window is closed
// }
// }
