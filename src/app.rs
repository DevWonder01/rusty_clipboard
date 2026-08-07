use iced::font::Family;
use iced::widget::{
    button, column, container, image, row, scrollable, text, text_input, Space,
};
use iced::{
    alignment, Border, Color, Element, Font, Length, Padding, Subscription, Task, Vector,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::catalog::{get_emojis, get_kaomojis, get_symbols};
use crate::clipboard_listener::copy_item_to_clipboard;
use crate::db::ClipboardDb;
use crate::types::{
    ActiveTab, ClipboardItem, EmojiCategory, ItemType, KaomojiCategory, SymbolCategory,
};

pub const SYSTEM_FONT: Font = Font {
    family: Family::SansSerif,
    weight: iced::font::Weight::Normal,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

pub const EMOJI_FONT: Font = Font {
    family: Family::Name("Noto Color Emoji"),
    weight: iced::font::Weight::Normal,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

#[derive(Debug, Clone)]
pub enum Message {
    SelectTab(ActiveTab),
    SearchChanged(String),
    SelectEmojiCat(EmojiCategory),
    SelectSymbolCat(SymbolCategory),
    SelectKaomojiCat(KaomojiCategory),
    CopyText(String),
    CopyItem(ClipboardItem),
    TogglePin(String),
    DeleteItem(String),
    ClearUnpinned,
    FontLoaded(Result<(), iced::font::Error>),
    Tick,
}

pub struct RustyClipboardApp {
    db: Arc<ClipboardDb>,
    receiver: crossbeam_channel::Receiver<ClipboardItem>,
    items: Vec<ClipboardItem>,
    active_tab: ActiveTab,
    search_query: String,
    toast_message: Option<(String, Instant)>,
    selected_emoji_cat: EmojiCategory,
    selected_symbol_cat: SymbolCategory,
    selected_kaomoji_cat: KaomojiCategory,
    image_handles: HashMap<String, image::Handle>,
}

impl RustyClipboardApp {
    pub fn new(
        db: Arc<ClipboardDb>,
        receiver: crossbeam_channel::Receiver<ClipboardItem>,
    ) -> (Self, Task<Message>) {
        let initial_items = db.get_all_items().unwrap_or_default();
        let mut app = Self {
            db,
            receiver,
            items: initial_items,
            active_tab: ActiveTab::Clipboard,
            search_query: String::new(),
            toast_message: None,
            selected_emoji_cat: EmojiCategory::Smileys,
            selected_symbol_cat: SymbolCategory::Currency,
            selected_kaomoji_cat: KaomojiCategory::Happy,
            image_handles: HashMap::new(),
        };

        app.build_image_handles();

        // Load system Noto Emoji & Symbols TTF fonts into Iced
        let font_paths = [
            "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf",
            "/usr/share/fonts/truetype/noto/NotoSansSymbols-Regular.ttf",
            "/usr/share/fonts/truetype/noto/NotoSansSymbols2-Regular.ttf",
        ];

        let mut tasks = Vec::new();
        for path in font_paths {
            if let Ok(bytes) = std::fs::read(path) {
                tasks.push(iced::font::load(bytes).map(Message::FontLoaded));
            }
        }

        (app, Task::batch(tasks))
    }

    fn build_image_handles(&mut self) {
        for item in &self.items {
            if let ItemType::Image {
                width,
                height,
                rgba_bytes,
            } = &item.item_type
            {
                if !self.image_handles.contains_key(&item.id) {
                    let handle = image::Handle::from_rgba(*width, *height, rgba_bytes.clone());
                    self.image_handles.insert(item.id.clone(), handle);
                }
            }
        }
    }

    fn show_toast(&mut self, msg: impl Into<String>) {
        self.toast_message = Some((msg.into(), Instant::now()));
    }

    fn refresh_items(&mut self) {
        if let Ok(latest) = self.db.get_all_items() {
            self.items = latest;
            self.build_image_handles();
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectTab(tab) => {
                self.active_tab = tab;
            }
            Message::SearchChanged(query) => {
                self.search_query = query;
            }
            Message::SelectEmojiCat(cat) => {
                self.selected_emoji_cat = cat;
            }
            Message::SelectSymbolCat(cat) => {
                self.selected_symbol_cat = cat;
            }
            Message::SelectKaomojiCat(cat) => {
                self.selected_kaomoji_cat = cat;
            }
            Message::CopyText(txt) => {
                if crate::clipboard_listener::copy_text_to_clipboard(&txt).is_ok() {
                    let preview: String = txt.chars().take(20).collect();
                    self.show_toast(format!("Copied \"{}\"", preview));
                }
            }
            Message::CopyItem(item) => {
                if copy_item_to_clipboard(&item).is_ok() {
                    self.show_toast("Copied item to clipboard!");
                }
            }
            Message::TogglePin(id) => {
                let _ = self.db.toggle_pin(&id);
                self.refresh_items();
            }
            Message::DeleteItem(id) => {
                let _ = self.db.delete_item(&id);
                self.refresh_items();
                self.show_toast("Item deleted");
            }
            Message::ClearUnpinned => {
                let _ = self.db.clear_unpinned();
                self.refresh_items();
                self.show_toast("Cleared unpinned history");
            }
            Message::FontLoaded(_) => {}
            Message::Tick => {
                let mut new_received = false;
                while let Ok(item) = self.receiver.try_recv() {
                    let _ = self.db.save_item(&item);
                    new_received = true;
                }
                if new_received {
                    self.refresh_items();
                }
            }
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_millis(300)).map(|_| Message::Tick)
    }

    pub fn view(&self) -> Element<'_, Message> {
        // Windows 11 Top handle bar
        let handle_bar = container(Space::with_height(0))
            .width(Length::Fixed(40.0))
            .height(Length::Fixed(4.0))
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.7, 0.7, 0.73).into()),
                border: Border {
                    radius: 2.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

        let top_drag = container(handle_bar)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .padding(Padding { top: 6.0, right: 0.0, bottom: 0.0, left: 0.0 });

        // Navigation Tabs Header (GIFs tab removed per request)
        let tabs = [
            (ActiveTab::Favorites, "📌 Pinned"),
            (ActiveTab::Emojis, "😊 Emojis"),
            (ActiveTab::Kaomoji, ";-) Kaomoji"),
            (ActiveTab::Symbols, "½¼ Symbols"),
            (ActiveTab::Clipboard, "📋 Clipboard"),
        ];

        let mut tab_row = row![].spacing(4).align_y(alignment::Vertical::Center);
        for (tab, label) in tabs {
            let is_sel = self.active_tab == tab;
            let tab_btn = button(
                text(label)
                    .font(SYSTEM_FONT)
                    .size(12)
                    .color(if is_sel { Color::from_rgb(0.0, 0.4, 0.75) } else { Color::from_rgb(0.3, 0.3, 0.35) }),
            )
            .padding([5.0, 8.0])
            .style(move |_, status| {
                let bg = if is_sel {
                    Color::from_rgb(0.9, 0.92, 0.96)
                } else if status == button::Status::Hovered {
                    Color::from_rgb(0.95, 0.95, 0.97)
                } else {
                    Color::TRANSPARENT
                };
                button::Style {
                    background: Some(bg.into()),
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
            .on_press(Message::SelectTab(tab));

            tab_row = tab_row.push(tab_btn);
        }

        let nav_bar = container(
            scrollable(tab_row)
                .direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::default()))
        )
        .padding([2.0, 4.0])
        .width(Length::Fill);

        // Search Input
        let search_bar = text_input("Search history & items...", &self.search_query)
            .on_input(Message::SearchChanged)
            .padding([8.0, 12.0])
            .font(SYSTEM_FONT)
            .size(13)
            .style(|_, _| text_input::Style {
                background: Color::WHITE.into(),
                border: Border {
                    radius: 6.0.into(),
                    width: 1.0,
                    color: Color::from_rgb(0.85, 0.85, 0.88),
                },
                value: Color::BLACK,
                placeholder: Color::from_rgb(0.6, 0.6, 0.6),
                selection: Color::from_rgb(0.7, 0.85, 1.0),
                icon: Color::from_rgb(0.5, 0.5, 0.5),
            });

        // Content Area
        let content: Element<Message> = match self.active_tab {
            ActiveTab::Clipboard => self.view_clipboard_history(),
            ActiveTab::Favorites => self.view_favorites(),
            ActiveTab::Emojis => self.view_emojis(),
            ActiveTab::Kaomoji => self.view_kaomoji(),
            ActiveTab::Symbols => self.view_symbols(),
        };

        let mut main_col = column![
            top_drag,
            Space::with_height(4),
            nav_bar,
            Space::with_height(6),
            container(search_bar).padding(Padding { top: 0.0, right: 12.0, bottom: 0.0, left: 12.0 }),
            Space::with_height(10),
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(Padding { top: 0.0, right: 12.0, bottom: 0.0, left: 12.0 })
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        // Toast at bottom
        if let Some((msg, created_at)) = &self.toast_message {
            if created_at.elapsed() < Duration::from_secs(2) {
                let toast_box = container(
                    text(format!("✓ {}", msg))
                        .font(SYSTEM_FONT)
                        .size(12)
                        .color(Color::WHITE),
                )
                .padding(Padding { top: 8.0, right: 12.0, bottom: 8.0, left: 12.0 })
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.15, 0.15, 0.18).into()),
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                });
                main_col = main_col.push(container(toast_box).padding(8.0));
            }
        }

        container(main_col)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.95, 0.95, 0.97).into()),
                ..Default::default()
            })
            .into()
    }

    fn view_clipboard_history(&self) -> Element<'_, Message> {
        let header_row = row![
            text("Clipboard")
                .font(SYSTEM_FONT)
                .size(16)
                .color(Color::from_rgb(0.15, 0.15, 0.18)),
            Space::with_width(Length::Fill),
            button(text("Clear all").font(SYSTEM_FONT).size(12).color(Color::from_rgb(0.25, 0.25, 0.28)))
                .padding([6.0, 12.0])
                .style(|_, _| button::Style {
                    background: Some(Color::WHITE.into()),
                    border: Border {
                        radius: 6.0.into(),
                        width: 1.0,
                        color: Color::from_rgb(0.85, 0.85, 0.88),
                    },
                    ..Default::default()
                })
                .on_press(Message::ClearUnpinned)
        ]
        .align_y(alignment::Vertical::Center);

        let query = self.search_query.trim().to_lowercase();
        let filtered: Vec<_> = self
            .items
            .iter()
            .filter(|item| {
                if query.is_empty() {
                    return true;
                }
                match &item.item_type {
                    ItemType::Text(t) => t.to_lowercase().contains(&query),
                    ItemType::Image { .. } => "image".contains(&query),
                }
            })
            .collect();

        if filtered.is_empty() {
            let empty_view = column![
                Space::with_height(40),
                text("📋").font(SYSTEM_FONT).size(32),
                Space::with_height(8),
                text("Your clipboard history is empty")
                    .font(SYSTEM_FONT)
                    .size(14)
                    .color(Color::from_rgb(0.4, 0.4, 0.45)),
                text("Copied text and images will appear here")
                    .font(SYSTEM_FONT)
                    .size(12)
                    .color(Color::from_rgb(0.6, 0.6, 0.65)),
            ]
            .align_x(alignment::Horizontal::Center)
            .width(Length::Fill);

            return column![header_row, Space::with_height(10), empty_view].into();
        }

        let mut cards_col = column![].spacing(10);
        for item in filtered {
            cards_col = cards_col.push(self.render_card(item));
        }

        column![
            header_row,
            Space::with_height(10),
            scrollable(cards_col).height(Length::Fill)
        ]
        .into()
    }

    fn view_favorites(&self) -> Element<'_, Message> {
        let pinned_items: Vec<_> = self.items.iter().filter(|i| i.pinned).collect();

        if pinned_items.is_empty() {
            let empty_view = column![
                Space::with_height(40),
                text("📌").font(SYSTEM_FONT).size(32),
                Space::with_height(8),
                text("No pinned items yet")
                    .font(SYSTEM_FONT)
                    .size(14)
                    .color(Color::from_rgb(0.4, 0.4, 0.45)),
                text("Pin items in Clipboard tab to keep them safe")
                    .font(SYSTEM_FONT)
                    .size(12)
                    .color(Color::from_rgb(0.6, 0.6, 0.65)),
            ]
            .align_x(alignment::Horizontal::Center)
            .width(Length::Fill);

            return empty_view.into();
        }

        let mut cards_col = column![].spacing(10);
        for item in pinned_items {
            cards_col = cards_col.push(self.render_card(item));
        }

        scrollable(cards_col).height(Length::Fill).into()
    }

    fn render_card<'a>(&'a self, item: &'a ClipboardItem) -> Element<'a, Message> {
        let content_element: Element<'a, Message> = match &item.item_type {
            ItemType::Text(t) => {
                let lines: Vec<&str> = t.lines().take(4).collect();
                let snippet = lines.join("\n");
                text(snippet)
                    .font(SYSTEM_FONT)
                    .size(13)
                    .color(Color::from_rgb(0.15, 0.15, 0.18))
                    .into()
            }
            ItemType::Image { width, height, .. } => {
                if let Some(handle) = self.image_handles.get(&item.id) {
                    column![
                        image(handle.clone()).width(Length::Fixed(240.0)),
                        Space::with_height(4),
                        text(format!("Image ({} × {})", width, height))
                            .font(SYSTEM_FONT)
                            .size(11)
                            .color(Color::from_rgb(0.5, 0.5, 0.5))
                    ]
                    .into()
                } else {
                    text(format!("[Image {}×{}]", width, height))
                        .font(SYSTEM_FONT)
                        .size(13)
                        .into()
                }
            }
        };

        let pin_color = if item.pinned {
            Color::from_rgb(0.0, 0.4, 0.75)
        } else {
            Color::from_rgb(0.6, 0.6, 0.65)
        };

        let card_body = column![
            row![
                container(content_element).width(Length::Fill),
                button(text("🗑").font(SYSTEM_FONT).size(12))
                    .padding(4.0)
                    .style(|_, _| button::Style {
                        background: Some(Color::TRANSPARENT.into()),
                        ..Default::default()
                    })
                    .on_press(Message::DeleteItem(item.id.clone()))
            ],
            Space::with_height(6),
            row![
                Space::with_width(Length::Fill),
                button(text("📌").font(SYSTEM_FONT).size(14).color(pin_color))
                    .padding(4.0)
                    .style(|_, _| button::Style {
                        background: Some(Color::TRANSPARENT.into()),
                        ..Default::default()
                    })
                    .on_press(Message::TogglePin(item.id.clone()))
            ]
        ];

        let item_clone = item.clone();
        button(card_body)
            .width(Length::Fill)
            .padding(12.0)
            .style(|_, status| button::Style {
                background: Some(Color::WHITE.into()),
                border: Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: if status == button::Status::Hovered {
                        Color::from_rgb(0.0, 0.4, 0.75)
                    } else {
                        Color::from_rgb(0.88, 0.88, 0.9)
                    },
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.04),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 4.0,
                },
                ..Default::default()
            })
            .on_press(Message::CopyItem(item_clone))
            .into()
    }

    fn view_emojis(&self) -> Element<'_, Message> {
        let categories = [
            (EmojiCategory::Smileys, "Smileys"),
            (EmojiCategory::Animals, "Animals"),
            (EmojiCategory::Food, "Food"),
            (EmojiCategory::Activities, "Sports"),
            (EmojiCategory::Travel, "Travel"),
            (EmojiCategory::Objects, "Objects"),
            (EmojiCategory::Symbols, "Symbols"),
        ];

        let mut cat_row = row![].spacing(4);
        for (cat, label) in categories {
            let is_sel = self.selected_emoji_cat == cat;
            let cat_btn = button(
                text(label)
                    .font(SYSTEM_FONT)
                    .size(11)
                    .color(if is_sel {
                        Color::from_rgb(0.0, 0.4, 0.75)
                    } else {
                        Color::from_rgb(0.3, 0.3, 0.35)
                    }),
            )
            .padding([4.0, 8.0])
            .style(move |_, _| button::Style {
                background: Some(
                    if is_sel {
                        Color::from_rgb(0.9, 0.92, 0.96)
                    } else {
                        Color::TRANSPARENT
                    }
                    .into(),
                ),
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .on_press(Message::SelectEmojiCat(cat));

            cat_row = cat_row.push(cat_btn);
        }

        let query = self.search_query.trim().to_lowercase();
        let emojis: Vec<_> = get_emojis()
            .into_iter()
            .filter(|e| e.category == self.selected_emoji_cat)
            .filter(|e| query.is_empty() || e.name.contains(&query) || e.char.contains(&query))
            .collect();

        let mut grid_row = row![].spacing(6);
        let mut grid_col = column![].spacing(6);

        for (idx, e) in emojis.into_iter().enumerate() {
            let e_btn = button(text(e.char).font(EMOJI_FONT).size(20))
                .padding(6.0)
                .style(|_, status| button::Style {
                    background: Some(
                        if status == button::Status::Hovered {
                            Color::from_rgb(0.92, 0.94, 0.98)
                        } else {
                            Color::WHITE
                        }
                        .into(),
                    ),
                    border: Border {
                        radius: 6.0.into(),
                        width: 1.0,
                        color: Color::from_rgb(0.88, 0.88, 0.9),
                    },
                    ..Default::default()
                })
                .on_press(Message::CopyText(e.char.to_string()));

            grid_row = grid_row.push(e_btn);
            if (idx + 1) % 6 == 0 {
                grid_col = grid_col.push(grid_row);
                grid_row = row![].spacing(6);
            }
        }
        grid_col = grid_col.push(grid_row);

        column![
            scrollable(cat_row).direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::default())),
            Space::with_height(8),
            scrollable(grid_col).height(Length::Fill)
        ]
        .into()
    }

    fn view_kaomoji(&self) -> Element<'_, Message> {
        let categories = [
            (KaomojiCategory::Happy, "Happy"),
            (KaomojiCategory::Shrug, "Shrug"),
            (KaomojiCategory::Angry, "Angry"),
            (KaomojiCategory::Surprised, "Surprised"),
            (KaomojiCategory::Sad, "Sad"),
            (KaomojiCategory::Love, "Love"),
        ];

        let mut cat_row = row![].spacing(4);
        for (cat, label) in categories {
            let is_sel = self.selected_kaomoji_cat == cat;
            let cat_btn = button(
                text(label)
                    .font(SYSTEM_FONT)
                    .size(11)
                    .color(if is_sel {
                        Color::from_rgb(0.0, 0.4, 0.75)
                    } else {
                        Color::from_rgb(0.3, 0.3, 0.35)
                    }),
            )
            .padding([4.0, 8.0])
            .style(move |_, _| button::Style {
                background: Some(
                    if is_sel {
                        Color::from_rgb(0.9, 0.92, 0.96)
                    } else {
                        Color::TRANSPARENT
                    }
                    .into(),
                ),
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .on_press(Message::SelectKaomojiCat(cat));

            cat_row = cat_row.push(cat_btn);
        }

        let kaomojis: Vec<_> = get_kaomojis()
            .into_iter()
            .filter(|k| k.category == self.selected_kaomoji_cat)
            .collect();

        let mut grid_row = row![].spacing(8);
        let mut grid_col = column![].spacing(8);

        for (idx, k) in kaomojis.into_iter().enumerate() {
            let k_btn = button(
                text(k.text)
                    .font(SYSTEM_FONT)
                    .size(12)
                    .color(Color::from_rgb(0.2, 0.2, 0.22)),
            )
            .padding([8.0, 12.0])
            .style(|_, status| button::Style {
                background: Some(
                    if status == button::Status::Hovered {
                        Color::from_rgb(0.92, 0.94, 0.98)
                    } else {
                        Color::WHITE
                    }
                    .into(),
                ),
                border: Border {
                    radius: 6.0.into(),
                    width: 1.0,
                    color: Color::from_rgb(0.88, 0.88, 0.9),
                },
                ..Default::default()
            })
            .on_press(Message::CopyText(k.text.to_string()));

            grid_row = grid_row.push(k_btn);
            if (idx + 1) % 2 == 0 {
                grid_col = grid_col.push(grid_row);
                grid_row = row![].spacing(8);
            }
        }
        grid_col = grid_col.push(grid_row);

        column![
            scrollable(cat_row).direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::default())),
            Space::with_height(8),
            scrollable(grid_col).height(Length::Fill)
        ]
        .into()
    }

    fn view_symbols(&self) -> Element<'_, Message> {
        let categories = [
            (SymbolCategory::Currency, "Currency"),
            (SymbolCategory::Math, "Math"),
            (SymbolCategory::Punctuation, "Punctuation"),
            (SymbolCategory::Arrows, "Arrows"),
            (SymbolCategory::Greek, "Greek"),
        ];

        let mut cat_row = row![].spacing(4);
        for (cat, label) in categories {
            let is_sel = self.selected_symbol_cat == cat;
            let cat_btn = button(
                text(label)
                    .font(SYSTEM_FONT)
                    .size(11)
                    .color(if is_sel {
                        Color::from_rgb(0.0, 0.4, 0.75)
                    } else {
                        Color::from_rgb(0.3, 0.3, 0.35)
                    }),
            )
            .padding([4.0, 8.0])
            .style(move |_, _| button::Style {
                background: Some(
                    if is_sel {
                        Color::from_rgb(0.9, 0.92, 0.96)
                    } else {
                        Color::TRANSPARENT
                    }
                    .into(),
                ),
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .on_press(Message::SelectSymbolCat(cat));

            cat_row = cat_row.push(cat_btn);
        }

        let query = self.search_query.trim().to_lowercase();
        let symbols: Vec<_> = get_symbols()
            .into_iter()
            .filter(|s| s.category == self.selected_symbol_cat)
            .filter(|s| query.is_empty() || s.name.contains(&query) || s.symbol.contains(&query))
            .collect();

        let mut grid_row = row![].spacing(6);
        let mut grid_col = column![].spacing(6);

        for (idx, sym) in symbols.into_iter().enumerate() {
            let sym_btn = button(text(sym.symbol).font(SYSTEM_FONT).size(16))
                .padding(6.0)
                .style(|_, status| button::Style {
                    background: Some(
                        if status == button::Status::Hovered {
                            Color::from_rgb(0.92, 0.94, 0.98)
                        } else {
                            Color::WHITE
                        }
                        .into(),
                    ),
                    border: Border {
                        radius: 6.0.into(),
                        width: 1.0,
                        color: Color::from_rgb(0.88, 0.88, 0.9),
                    },
                    ..Default::default()
                })
                .on_press(Message::CopyText(sym.symbol.to_string()));

            grid_row = grid_row.push(sym_btn);
            if (idx + 1) % 6 == 0 {
                grid_col = grid_col.push(grid_row);
                grid_row = row![].spacing(6);
            }
        }
        grid_col = grid_col.push(grid_row);

        column![
            scrollable(cat_row).direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::default())),
            Space::with_height(8),
            scrollable(grid_col).height(Length::Fill)
        ]
        .into()
    }
}
