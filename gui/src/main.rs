//! Records a WAV file (roughly 3 seconds long) using the default input device and config.
//!
//! The input data is recorded to "$CARGO_MANIFEST_DIR/recorded.wav".

use clap::Parser;

use gtk::traits::{ButtonExt, GtkWindowExt};
use gtk::{prelude::*, TextView};
use gtk::{ApplicationWindow, Button};
use gwhisper::recogntion::Recognition;
use gwhisper::recording::{self, Recorder};
use std::rc::Rc;
use std::sync::Mutex;

const APP_NAME: &str = "gwhisper";
const APP_ID: &str = "com.marmistrz.GWhisper";

#[derive(Parser, Debug)]
#[command(version, about = APP_NAME, long_about = None)]
struct Opt {
    /// The audio device to use
    #[arg(short, long, default_value_t = String::from("default"))]
    device: String,
}

fn main() -> gtk::glib::ExitCode {
    // Set up the input device and stream with the default input config.
    let (device, config) = recording::whisper_config("default").expect("FIXME");

    // Create a new application
    let app = gtk::Application::builder().application_id(APP_ID).build();
    let res = Application {
        recorder: Rc::new(Mutex::new(Recorder::new(device, config.into()))),
        recognition: Rc::new(
            Recognition::new("/usr/share/whisper.cpp-model-base.en/base.en.bin").expect("FIXME"),
        ), // FIXME
    };

    // Connect to "activate" signal of `app`
    app.connect_activate(move |app| res.build_ui(app));

    // Run the application
    app.run()
}

struct Application {
    recorder: Rc<Mutex<Recorder>>,
    recognition: Rc<Recognition>,
}

impl Application {
    fn build_ui(&self, app: &gtk::Application) {
        let button = Button::builder()
            .label("Press me!")
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();
        let button = Rc::new(button);

        let text_view = TextView::builder()
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .valign(gtk::Align::Fill)
            .editable(true)
            .wrap_mode(gtk::WrapMode::Word)
            .can_focus(true)
            .focus_on_click(true)
            .expand(true)
            .build();
        let text_view = Rc::new(text_view);

        let layout = gtk::Box::builder()
            .expand(true)
            .orientation(gtk::Orientation::Vertical)
            .build();
        layout.pack_start(text_view.as_ref(), true, true, 0);
        layout.pack_end(button.as_ref(), false, false, 0);

        // Connect to "clicked" signal of `button`
        button.connect_clicked({
            let button = button.clone();
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
                    let buffer = text_view.buffer().expect("buffer");
                    let mut end = buffer.end_iter();
                    buffer.insert(&mut end, &text);
                }
            }
        });

        // Create a window
        let window = ApplicationWindow::builder()
            .application(app)
            .title(APP_NAME)
            .child(&layout)
            .default_height(600)
            .default_width(800)
            .build();

        // Present window
        window.show_all();
        window.present();
    }
}
