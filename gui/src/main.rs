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
use relm::{connect, Update, Widget, Channel, Sender};
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
    if gtk::init().is_err() {
        panic!("Failed to initialize GTK.");
    }
    App::run(()).expect("run failed");
    gtk::main()
}

struct Resources {
    recorder: Rc<Mutex<Recorder>>,
    recognition: Arc<Mutex<Option<Recognition>>>,
}

struct Model {
    _channel: Channel<Msg>,
    text_sender: Sender<Msg>,
}

struct App {
    resources: Resources,
    ui: Ui,
    // model: Model,
}

#[derive(Msg)]
enum Msg {
    ToggleRecord,
    LoadModel,
    CopyText,
    SetLang,
    RecvText(String),
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

impl Ui {
    fn load_glade() -> Self {
        let glade_src = include_str!("gwhisper.glade");
        let builder = gtk::Builder::from_string(glade_src);

        let record_button = builder.object("recognition_button").unwrap();
        let copy_button = builder.object("copy_button").unwrap();
        let text_view = builder.object("text_view").unwrap();
        let window = builder.object("window").unwrap();
        let lang_combo_box = builder.object("lang_combo_box").unwrap();
        let model_label = builder.object("model_label").unwrap();
        let model_choice_button = builder.object("model_choice_button").unwrap();

        Self {
            record_button,
            text_view,
            window,
            lang_combo_box,
            copy_button,
            model_label,
            model_choice_button,
        }
    }
}

impl App {
    fn toggle_record(&self) {
        let recorder = self.resources.recorder.clone();
        let recognition = self.resources.recognition.clone();
        let mut recorder = recorder.lock().unwrap();
        if recorder.is_stopped() {
            recorder.start().expect("FIXME");
            self.ui.record_button.set_label("Recording...");
        } else {
            self.ui.record_button.set_sensitive(false);
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

            //tx.send(text).expect("channel error");
            // }
        }
    }

    fn load_model(&self) {
        let dialog = FileChooserDialog::new(
            Some("Open model"),
            Some(&self.root()),
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
            let mut guard = self.resources.recognition.lock().unwrap();
            Resources::set_model(&mut *guard, &self.ui, &model);
        }
    }

    fn copy_text(&self) {
        let clipboard = Clipboard::get(&gdk::SELECTION_CLIPBOARD);
        let buffer = self.ui.text_view.buffer().expect("textview buffer");
        let text = buffer
            .text(&buffer.start_iter(), &buffer.end_iter(), true)
            .expect("buffer text");
        clipboard.set_text(text.as_str());
    }

    fn set_lang(&self) {
        let recognition = self.resources.recognition.clone();
        let lang = self
            .ui
            .lang_combo_box
            .active_text()
            .expect("should be selected");

        recognition
            .lock()
            .unwrap()
            .as_mut()
            .expect("model not initialized")
            .set_lang(lang.as_str());
    }
}

impl Update for App {
    type Model = (); // FIXME properly use the model

    type ModelParam = ();

    type Msg = Msg;

    // TODO figure out how to use the model
    fn model(relm: &relm::Relm<Self>, _param: Self::ModelParam) -> Self::Model {
        let stream = relm.stream().clone();
        let (channel, sender) = Channel::new(move |text| {
            // This closure is executed whenever a message is received from the sender.
            // We send a message to the current widget.
            stream.emit(Msg::RecvText(text));
        });

        ()
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            Msg::ToggleRecord => self.toggle_record(),
            Msg::LoadModel => self.load_model(),
            Msg::CopyText => self.copy_text(),
            Msg::SetLang => self.set_lang(),
            Msg::RecvText(text) => todo!(),
        }
    }
}

impl Widget for App {
    type Root = ApplicationWindow;

    fn root(&self) -> Self::Root {
        self.ui.window.clone()
    }

    fn view(relm: &relm::Relm<Self>, _model: Self::Model) -> Self {
        let ui = Ui::load_glade();

        for lang in all_langs() {
            ui.lang_combo_box.append_text(lang);
            // TODO set default as active
        }

        ui.window.show_all();
        connect!(
            relm,
            ui.record_button,
            connect_clicked(_),
            Msg::ToggleRecord
        );
        connect!(
            relm,
            ui.model_choice_button,
            connect_clicked(_),
            Msg::LoadModel
        );
        connect!(relm, ui.copy_button, connect_clicked(_), Msg::CopyText);
        connect!(relm, ui.lang_combo_box, connect_changed(_), Msg::SetLang);

        let resources = Resources::default();
        Self { ui, resources }
    }
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
    }*/
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn ui_labels() {
        if gtk::init().is_err() {
            panic!("Failed to initialize GTK.");
        }
        let _ = Ui::load_glade();
    }
}
