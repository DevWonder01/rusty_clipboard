use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ItemType {
    Text(String),
    Image {
        width: u32,
        height: u32,
        rgba_bytes: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClipboardItem {
    pub id: String,
    pub timestamp: i64,
    pub item_type: ItemType,
    pub pinned: bool,
}

impl ClipboardItem {
    pub fn new_text(text: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            item_type: ItemType::Text(text),
            pinned: false,
        }
    }

    pub fn new_image(width: u32, height: u32, rgba_bytes: Vec<u8>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            item_type: ItemType::Image {
                width,
                height,
                rgba_bytes,
            },
            pinned: false,
        }
    }

    pub fn preview_text(&self) -> String {
        match &self.item_type {
            ItemType::Text(t) => t.clone(),
            ItemType::Image { width, height, .. } => format!("[Image {}x{}]", width, height),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActiveTab {
    Clipboard,
    Favorites,
    Emojis,
    Symbols,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmojiCategory {
    Smileys,
    Animals,
    Food,
    Activities,
    Travel,
    Objects,
    Symbols,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolCategory {
    Currency,
    Math,
    Punctuation,
    Arrows,
    Greek,
}
