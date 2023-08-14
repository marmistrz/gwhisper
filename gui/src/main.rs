//! Records a WAV file (roughly 3 seconds long) using the default input device and config.
//!
//! The input data is recorded to "$CARGO_MANIFEST_DIR/recorded.wav".

use clap::Parser;

use gtk::traits::ButtonExt;
use gtk::{prelude::*, Button, Clipboard, TextView};
use gwhisper::recogntion::{Recognition, all_langs};
use gwhisper::recording::{self, Recorder};
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
    let recognition =
        Recognition::new("/home/marcin/build/whisper.cpp/models/ggml-medium.bin").expect("FIXME");
    let app = Application {
        recorder: Rc::new(Mutex::new(recorder)),
        recognition: Arc::new(Mutex::new(recognition)),
    };

    if gtk::init().is_err() {
        panic!("Failed to initialize GTK.");
    }
    app.setup();
    gtk::main()
}

struct Application {
    recorder: Rc<Mutex<Recorder>>,
    recognition: Arc<Mutex<Recognition>>,
}

struct Ui {
    button: Rc<Button>,
    copy_button: Rc<Button>,
    text_view: Rc<TextView>,
    window: gtk::Window,
    lang_combo_box: gtk::ComboBoxText,
}

impl Default for Ui {
    fn default() -> Self {
        let glade_src = include_str!("gwhisper.glade");
        let builder = gtk::Builder::from_string(glade_src);

        let button: Button = builder.object("recognition_button").unwrap();
        let button = Rc::new(button);

        let copy_button: Button = builder.object("copy_button").unwrap();
        let copy_button = Rc::new(copy_button);

        let text_view: TextView = builder.object("text_view").unwrap();
        let text_view = Rc::new(text_view);

        let window: gtk::Window = builder.object("window").unwrap();
        let lang_combo_box = builder.object("lang_combo_box").unwrap();

        Self {
            button,
            text_view,
            window,
            lang_combo_box,
            copy_button,
        }
    }
}

impl Application {
    fn setup(&self) {
        let ui = Ui::default();
        let (data_tx, data_rx) = glib::MainContext::channel(glib::Priority::DEFAULT);

        data_rx.attach(None, {
            let button = ui.button.clone();
            let text_view = ui.text_view.clone();
            move |text: String| {
                button.set_sensitive(true);
                let buffer = text_view.buffer().expect("buffer");
                let mut end = buffer.end_iter();
                buffer.insert(&mut end, &text);
                button.set_label("Record");

                glib::ControlFlow::Continue
            }
        });

        // Connect to "clicked" signal of `button`
        ui.button.connect_clicked({
            let button = ui.button.clone();
            let recorder = self.recorder.clone();
            let recognition = self.recognition.clone();
            move |_| {
                let mut recorder = recorder.lock().unwrap();
                if recorder.is_stopped() {
                    recorder.start().expect("FIXME");
                    button.set_label("Recording...");
                } else {
                    button.set_sensitive(false);
                    let audio = recorder.stop();
                    // TODO progress bar, but it requires extern C callbacks
                    thread::spawn({
                        let recognition = recognition.clone();
                        let tx = data_tx.clone();
                        move || {
                            let text = recognition.lock().unwrap().recognize(&audio);
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
                recognition.lock().unwrap().set_lang(lang.as_str());
            }
        });

        let clipboard = Clipboard::get(&gdk::SELECTION_CLIPBOARD);
        ui.copy_button.connect_clicked({
            let text_view = ui.text_view.clone();
            move |_| {
                let buffer = text_view.buffer().expect("textview buffer");
                let text = buffer
                    .text(&buffer.start_iter(), &buffer.end_iter(), true)
                    .expect("buffer text");
                clipboard.set_text(text.as_str());
            }
        });

        // Present window
        ui.window.show_all();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn ui_labels() {
        if gtk::init().is_err() {
            panic!("Failed to initialize GTK.");
        }
        let _ = Ui::default();
    }
}
