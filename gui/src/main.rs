//! Records a WAV file (roughly 3 seconds long) using the default input device and config.
//!
//! The input data is recorded to "$CARGO_MANIFEST_DIR/recorded.wav".

use clap::Parser;

use gtk::traits::ButtonExt;
use gtk::{prelude::*, Button, TextView};
use gwhisper::recogntion::Recognition;
use gwhisper::recording::{self, Recorder};
use std::rc::Rc;
use std::sync::Mutex;

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

    // Create a new application
    let app = Application {
        recorder: Rc::new(Mutex::new(Recorder::new(device, config.into()))),
        recognition: Rc::new(
            Recognition::new("/usr/share/whisper.cpp-model-base.en/base.en.bin").expect("FIXME"),
        ), // FIXME
    };

    if gtk::init().is_err() {
        panic!("Failed to initialize GTK.");
    }
    app.setup();
    gtk::main()
}

struct Application {
    recorder: Rc<Mutex<Recorder>>,
    recognition: Rc<Recognition>,
}

struct Ui {
    button: Rc<Button>,
    text_view: Rc<TextView>,
    window: gtk::Window,
}

impl Default for Ui {
    fn default() -> Self {
        let glade_src = include_str!("gwhisper.glade");
        let builder = gtk::Builder::from_string(glade_src);

        let button: Button = builder.object("recognition_button").unwrap();
        let button = Rc::new(button);

        let text_view: TextView = builder.object("text_view").unwrap();
        let text_view = Rc::new(text_view);

        let window: gtk::Window = builder.object("window").unwrap();

        Self {
            button,
            text_view,
            window,
        }
    }
}

impl Application {
    fn setup(&self) {
        let ui = Ui::default();
        // Connect to "clicked" signal of `button`
        ui.button.connect_clicked({
            let button = ui.button.clone();
            let recorder = self.recorder.clone();
            let recognition = self.recognition.clone();
            move |_| {
                let mut recorder = recorder.lock().unwrap();
                if recorder.is_stopped() {
                    recorder.start().expect("FIXME");
                    button.set_label("Recording");
                } else {
                    let audio = recorder.stop();
                    let text = recognition.recognize(&audio, "en");
                    let buffer = ui.text_view.buffer().expect("buffer");
                    let mut end = buffer.end_iter();
                    buffer.insert(&mut end, &text);
                }
            }
        });

        // Create a window

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
