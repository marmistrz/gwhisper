//! Records a WAV file (roughly 3 seconds long) using the default input device and config.
//!
//! The input data is recorded to "$CARGO_MANIFEST_DIR/recorded.wav".

use clap::Parser;

use gtk::traits::ButtonExt;
use gtk::{
    prelude::*, ApplicationWindow, Button, Clipboard, FileChooserDialog, ResponseType, TextView,
};
use gwhisper::recogntion::{all_langs, Recognition};
use gwhisper::recording::{self, Recorder};
use relm::{connect, Update, Widget};
use relm_derive::Msg;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

const APP_NAME: &str = "gwhisper";

#[derive(Parser, Debug)]
#[command(version, about = APP_NAME, long_about = None)]
struct Opt {
    /// The audio device to use
    #[arg(short, long, default_value_t = String::from("default"))]
    device: String,
}

fn main() {
    // Set up the input device and stream with the default input config.
    let (device, config) = recording::whisper_config("default").expect("FIXME");

    let recorder = Recorder::new(device, config.into());
    let app = Resources {
        recorder: Rc::new(Mutex::new(recorder)),
        recognition: Arc::new(Mutex::new(None)),
    };

    if gtk::init().is_err() {
        panic!("Failed to initialize GTK.");
    }
    Ui::run(()).expect("run failed");
    gtk::main()
}

struct Resources {
    recorder: Rc<Mutex<Recorder>>,
    recognition: Arc<Mutex<Option<Recognition>>>,
}

struct App {
    resources: Resources,
    ui: Ui,
}

#[derive(Msg)]
enum Msg {
    ToggleRecord,
    // Quit,
}

#[derive(Clone)]
struct Ui {
    record_button: Button,
    copy_button: Button,
    text_view: TextView,
    window: ApplicationWindow,
    lang_combo_box: gtk::ComboBoxText,
    model_label: gtk::Label,
    model_choice_button: gtk::Button,
}

impl Update for App {
    type Model = (); // FIXME

    type ModelParam = (); // FIXME

    type Msg = Msg; // FIXME

    fn model(relm: &relm::Relm<Self>, param: Self::ModelParam) -> Self::Model {
        () // FIXME
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            Msg::ToggleRecord => {
            let recorder = self.recorder.clone();
            let recognition = self.recognition.clone();
            move |_| {
                let mut recorder = recorder.lock().unwrap();
                if recorder.is_stopped() {
                    recorder.start().expect("FIXME");
                    ui.record_button.set_label("Recording...");
                } else {
                    ui.record_button.set_sensitive(false);
                    let audio = recorder.stop();
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
                            //tx.send(text).expect("channel error");
                        // }
                    }
                }
            }
        }
    }
}

impl Widget for App {
    type Root = ApplicationWindow;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &relm::Relm<Self>, model: Self::Model) -> Self {
        let glade_src = include_str!("gwhisper.glade");
        let builder = gtk::Builder::from_string(glade_src);

        let record_button = builder.object("recognition_button").unwrap();
        let copy_button = builder.object("copy_button").unwrap();
        let text_view = builder.object("text_view").unwrap();
        let window = builder.object("window").unwrap();
        let lang_combo_box = builder.object("lang_combo_box").unwrap();
        let model_label = builder.object("model_label").unwrap();
        let model_choice_button = builder.object("model_choice_button").unwrap();

        let ui = Self {
            record_button,
            text_view,
            window,
            lang_combo_box,
            copy_button,
            model_label,
            model_choice_button,
        };

        ui.window.show_all();
        connect!(
            relm,
            ui.record_button,
            connect_clicked(_),
            Msg::ToggleRecord
        );

        ui
    }
}

impl Resources {
    fn set_model(recognition: &mut Option<Recognition>, ui: &Ui, model: &Path) {
        let path = model.to_str().expect("invalid utf8");

        match Recognition::new(path) {
            Ok(rec) => {
                *recognition = Some(rec);
                ui.model_label.set_text(&format!("Model: {}", path));
                ui.record_button.set_sensitive(true);
            }
            Err(e) => {
                let dialog = gtk::MessageDialog::builder()
                    .parent(&ui.window)
                    .message_type(gtk::MessageType::Error)
                    .text(&e.to_string())
                    .buttons(gtk::ButtonsType::Ok)
                    .build();
                dialog.run();
                dialog.close();
            }
        }
    }

    /*fn setup(&self) {
        let ui = Rc::new(Ui::default());
        let (data_tx, data_rx) = glib::MainContext::channel(glib::Priority::DEFAULT);

        data_rx.attach(None, {
            let ui = ui.clone();
            move |text: String| {
                ui.record_button.set_sensitive(true);
                let buffer = ui.text_view.buffer().expect("buffer");
                let mut end = buffer.end_iter();
                buffer.insert(&mut end, &text);
                ui.record_button.set_label("Record");

                glib::ControlFlow::Continue
            }
        });

        ui.record_button.connect_clicked({
            let ui = ui.clone();
            let recorder = self.recorder.clone();
            let recognition = self.recognition.clone();
            move |_| {
                let mut recorder = recorder.lock().unwrap();
                if recorder.is_stopped() {
                    recorder.start().expect("FIXME");
                    ui.record_button.set_label("Recording...");
                } else {
                    ui.record_button.set_sensitive(false);
                    let audio = recorder.stop();
                    thread::spawn({
                        let recognition = recognition.clone();
                        let tx = data_tx.clone();
                        move || {
                            let text = recognition
                                .lock()
                                .unwrap()
                                .as_ref()
                                .expect("record button should be insensitive")
                                .recognize(&audio)
                                .expect("TODO: show a dialog for whisper error");
                            tx.send(text).expect("channel error");
                        }
                    });
                }
            }
        });

        for lang in all_langs() {
            ui.lang_combo_box.append_text(lang);
            // TODO set default as active
        }
        ui.lang_combo_box.connect_changed({
            let recognition = self.recognition.clone();
            move |combo| {
                let lang = combo.active_text().expect("should be selected");
                recognition
                    .lock()
                    .unwrap()
                    .as_mut()
                    .expect("model not initialized")
                    .set_lang(lang.as_str());
            }
        });

        let clipboard = Clipboard::get(&gdk::SELECTION_CLIPBOARD);
        ui.copy_button.connect_clicked({
            let ui = ui.clone();
            move |_| {
                let buffer = ui.text_view.buffer().expect("textview buffer");
                let text = buffer
                    .text(&buffer.start_iter(), &buffer.end_iter(), true)
                    .expect("buffer text");
                clipboard.set_text(text.as_str());
            }
        });

        ui.model_choice_button.connect_clicked({
            let recognition = self.recognition.clone();
            let ui = ui.clone();
            move |_| {
                let dialog = FileChooserDialog::new(
                    Some("Open model"),
                    Some(&ui.window),
                    gtk::FileChooserAction::Open,
                );
                dialog.add_button("OK", ResponseType::Accept);
                dialog.add_button("Cancel", ResponseType::Cancel);

                let resp = dialog.run();
                dialog.close(); // FIXME: the dialog is not really closed in case of an error. Perhaps idle_add or sth?
                if let ResponseType::Accept = resp {
                    let model = dialog
                        .filename()
                        .expect("TODO: when can the filename be none?");
                    let mut guard = recognition.lock().unwrap();
                    Self::set_model(&mut *guard, ui.as_ref(), &model);
                }
            }
        });

        // Present window
        ui.window.show_all();
        // TODO the executable should terminate when the window is closed
    }*/
}

#[cfg(test)]
mod test {
    use relm::EventStream;

    use super::*;

    #[test]
    pub fn ui_labels() {
        if gtk::init().is_err() {
            panic!("Failed to initialize GTK.");
        }
        let stream = EventStream::new();
        let relm = relm::Relm::new(&stream);
        let _ = Ui::view(&relm, ());
    }
}
