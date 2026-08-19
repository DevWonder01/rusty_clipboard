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

fn create_app_icon() -> Option<iced::window::Icon> {
    let width = 64;
    let height = 64;
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            let dx = if x < 8 { 8 - x } else if x >= 56 { x - 55 } else { 0 };
            let dy = if y < 8 { 8 - y } else if y >= 56 { y - 55 } else { 0 };
            let is_corner = (dx * dx + dy * dy) > 64;

            if is_corner {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
                continue;
            }

            // Top clip handle (y: 4..12, x: 24..40)
            if y >= 4 && y <= 12 && x >= 24 && x <= 40 {
                if y >= 8 && y <= 10 && x >= 30 && x <= 34 {
                    rgba.extend_from_slice(&[0x12, 0x12, 0x14, 0xFF]);
                } else {
                    rgba.extend_from_slice(&[0xF5, 0x9E, 0x0B, 0xFF]);
                }
                continue;
            }

            // Clipboard body (x: 12..52, y: 10..58)
            if x >= 12 && x <= 51 && y >= 10 && y <= 57 {
                if x == 12 || x == 51 || y == 10 || y == 57 {
                    rgba.extend_from_slice(&[0x3D, 0x3D, 0x47, 0xFF]);
                    continue;
                }

                if (y >= 20 && y <= 23) && x >= 20 && x <= 44 {
                    rgba.extend_from_slice(&[0xF5, 0xF5, 0xF7, 0xFF]);
                } else if (y >= 30 && y <= 33) && x >= 20 && x <= 38 {
                    rgba.extend_from_slice(&[0xA1, 0xA1, 0xAA, 0xFF]);
                } else if (y >= 40 && y <= 43) && x >= 20 && x <= 42 {
                    rgba.extend_from_slice(&[0xA1, 0xA1, 0xAA, 0xFF]);
                } else if (y >= 50 && y <= 52) && x >= 20 && x <= 32 {
                    rgba.extend_from_slice(&[0x71, 0x71, 0x7A, 0xFF]);
                } else {
                    rgba.extend_from_slice(&[0x1F, 0x1F, 0x24, 0xFF]);
                }
                continue;
            }

            rgba.extend_from_slice(&[0x12, 0x12, 0x14, 0xFF]);
        }
    }

    iced::window::icon::from_rgba(rgba, width, height).ok()
}

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

    // 3. Launch Iced GUI application with window icon and system sans-serif font
    iced::application(
        "Rusty Clipboard",
        RustyClipboardApp::update,
        RustyClipboardApp::view,
    )
    .window(iced::window::Settings {
        size: iced::Size::new(500.0, 620.0),
        icon: create_app_icon(),
        ..Default::default()
    })
    .default_font(Font {
        family: Family::SansSerif,
        ..Font::DEFAULT
    })
    .subscription(RustyClipboardApp::subscription)
    .run_with(move || RustyClipboardApp::new(db, receiver))
}
