mod app;
mod catalog;
mod clipboard_listener;
mod db;
mod types;

use app::RustyClipboardApp;
use clipboard_listener::ClipboardListener;
use db::ClipboardDb;
use iced::font::Family;
use iced::Font;
use std::sync::Arc;

fn main() -> iced::Result {
    // 1. Initialize local heed database
    let db = match ClipboardDb::init() {
        Ok(database) => Arc::new(database),
        Err(e) => {
            eprintln!("Failed to initialize heed database: {}", e);
            std::process::exit(1);
        }
    };

    // 2. Start background clipboard listener thread
    let (sender, receiver) = crossbeam_channel::unbounded();
    let _listener = ClipboardListener::start(sender);

    // 3. Launch Iced GUI application with System Sans-Serif font to enable system emoji & unicode font fallbacks
    iced::application(
        "Rusty Clipboard",
        RustyClipboardApp::update,
        RustyClipboardApp::view,
    )
    .default_font(Font {
        family: Family::SansSerif,
        ..Font::DEFAULT
    })
    .subscription(RustyClipboardApp::subscription)
    .window_size((360.0, 560.0))
    .run_with(move || RustyClipboardApp::new(db, receiver))
}
