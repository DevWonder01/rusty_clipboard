use arboard::Clipboard;
use crossbeam_channel::Sender;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::types::{ClipboardItem, ItemType};

pub struct ClipboardListener {
    stop_signal: Arc<AtomicBool>,
}

impl ClipboardListener {
    pub fn start(sender: Sender<ClipboardItem>) -> Self {
        let stop_signal = Arc::new(AtomicBool::new(false));
        let stop_flag = stop_signal.clone();

        thread::spawn(move || {
            let mut clipboard = match Clipboard::new() {
                Ok(cb) => cb,
                Err(e) => {
                    eprintln!("Failed to initialize clipboard listener: {}", e);
                    return;
                }
            };

            let mut last_text: Option<String> = None;
            let mut last_image_hash: Option<u64> = None;

            while !stop_flag.load(Ordering::Relaxed) {
                // Check text
                if let Ok(text) = clipboard.get_text() {
                    if !text.trim().is_empty() && last_text.as_ref() != Some(&text) {
                        last_text = Some(text.clone());
                        let item = ClipboardItem::new_text(text);
                        let _ = sender.send(item);
                    }
                }

                // Check image
                if let Ok(img) = clipboard.get_image() {
                    let mut hasher = DefaultHasher::new();
                    img.width.hash(&mut hasher);
                    img.height.hash(&mut hasher);
                    img.bytes.hash(&mut hasher);
                    let hash = hasher.finish();

                    if last_image_hash != Some(hash) {
                        last_image_hash = Some(hash);
                        let rgba_bytes = img.bytes.to_vec();
                        let item = ClipboardItem::new_image(
                            img.width as u32,
                            img.height as u32,
                            rgba_bytes,
                        );
                        let _ = sender.send(item);
                    }
                }

                thread::sleep(Duration::from_millis(400));
            }
        });

        Self { stop_signal }
    }
}

impl Drop for ClipboardListener {
    fn drop(&mut self) {
        self.stop_signal.store(true, Ordering::Relaxed);
    }
}

pub fn copy_item_to_clipboard(item: &ClipboardItem) -> Result<(), Box<dyn std::error::Error>> {
    let mut cb = Clipboard::new()?;
    match &item.item_type {
        ItemType::Text(text) => {
            cb.set_text(text)?;
        }
        ItemType::Image {
            width,
            height,
            rgba_bytes,
        } => {
            let img = arboard::ImageData {
                width: *width as usize,
                height: *height as usize,
                bytes: std::borrow::Cow::Borrowed(rgba_bytes),
            };
            cb.set_image(img)?;
        }
    }
    Ok(())
}

pub fn copy_text_to_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut cb = Clipboard::new()?;
    cb.set_text(text)?;
    Ok(())
}
