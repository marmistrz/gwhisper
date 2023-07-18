//! Records a WAV file (roughly 3 seconds long) using the default input device and config.
//!
//! The input data is recorded to "$CARGO_MANIFEST_DIR/recorded.wav".

use clap::Parser;

use gtk::traits::{ButtonExt, GtkWindowExt};
use gtk::{prelude::*, TextView};
use gtk::{Application, ApplicationWindow, Button};
use std::rc::Rc;

const APP_NAME: &str = "gwhisper";
const APP_ID: &str = "com.marmistrz.GWhisper";

#[derive(Parser, Debug)]
#[command(version, about = APP_NAME, long_about = None)]
struct Opt {
    /// The audio device to use
    #[arg(short, long, default_value_t = String::from("default"))]
    device: String,
}

// TODO use proper errors

fn main() -> gtk::glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    let button = Button::builder()
        .label("Press me!")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

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
    layout.pack_end(&button, false, false, 0);

    // Connect to "clicked" signal of `button`
    button.connect_clicked(move |_| {
        let voice = ""; // voice_recognition("default").expect("FIXME");
        text_view.buffer().expect("buffer").set_text(&voice);
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
