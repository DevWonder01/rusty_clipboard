use iced::font::Family;
use iced::widget::{
    button, column, container, image, row, scrollable, text, text_input, Space,
};
use iced::{
    alignment, Border, Color, Element, Font, Length, Padding, Subscription, Task, Vector,
};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::catalog::{get_emojis, get_symbols};
use crate::clipboard_listener::copy_item_to_clipboard;
use crate::db::ClipboardDb;
use crate::types::{
    ActiveTab, ClipboardItem, EmojiCategory, ItemType, SymbolCategory,
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

// --- Neutral Dark Theme Palette (Pure Onyx / Charcoal - No Blue) ---
const COLOR_BG_APP: Color = Color::from_rgb(0.07, 0.07, 0.08);         // #121214 Deep Onyx Dark Background
const COLOR_BG_CARD: Color = Color::from_rgb(0.12, 0.12, 0.14);        // #1F1F24 Card Neutral Surface
const COLOR_BG_CARD_HOVER: Color = Color::from_rgb(0.18, 0.18, 0.21);  // #2E2E36 Elevated Surface Hover
const COLOR_BORDER: Color = Color::from_rgb(0.20, 0.20, 0.23);         // #33333B Subtle Neutral Border
const COLOR_BORDER_HOVER: Color = Color::from_rgb(0.38, 0.38, 0.42);   // #61616B Active Neutral Border
const COLOR_ACCENT: Color = Color::from_rgb(0.24, 0.24, 0.28);         // #3D3D47 Neutral Active Tab Fill
const COLOR_TEXT_PRIMARY: Color = Color::from_rgb(0.96, 0.96, 0.97);   // #F5F5F7 Crisp Off-White Text
const COLOR_TEXT_SECONDARY: Color = Color::from_rgb(0.63, 0.63, 0.67); // #A1A1AA Neutral Light Gray
const COLOR_TEXT_MUTED: Color = Color::from_rgb(0.44, 0.44, 0.48);     // #71717A Muted Gray
const COLOR_PIN_ACTIVE: Color = Color::from_rgb(0.96, 0.62, 0.04);     // #F59E0B Warm Amber Gold
const COLOR_DANGER: Color = Color::from_rgb(0.94, 0.27, 0.27);         // #EF4444 Red Danger

#[derive(Debug, Clone)]
pub enum Message {
    SelectTab(ActiveTab),
    SearchChanged(String),
    SelectEmojiCat(EmojiCategory),
    SelectSymbolCat(SymbolCategory),
    CopyText(String),
    CopyItem(String),
    TogglePin(String),
    DeleteItem(String),
    ClearUnpinned,
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
}

impl RustyClipboardApp {
    pub fn new(
        db: Arc<ClipboardDb>,
        receiver: crossbeam_channel::Receiver<ClipboardItem>,
    ) -> (Self, Task<Message>) {
        let initial_items = db.get_all_items().unwrap_or_default();
        let app = Self {
            db,
            receiver,
            items: initial_items,
            active_tab: ActiveTab::Clipboard,
            search_query: String::new(),
            toast_message: None,
            selected_emoji_cat: EmojiCategory::Smileys,
            selected_symbol_cat: SymbolCategory::Currency,
        };

        (app, Task::none())
    }

    fn show_toast(&mut self, msg: impl Into<String>) {
        self.toast_message = Some((msg.into(), Instant::now()));
    }

    fn refresh_items(&mut self) {
        if let Ok(latest) = self.db.get_all_items() {
            self.items = latest;
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
            Message::CopyText(txt) => {
                if crate::clipboard_listener::copy_text_to_clipboard(&txt).is_ok() {
                    let preview: String = txt.chars().take(24).collect();
                    let display_text = if txt.chars().count() > 24 {
                        format!("{}...", preview)
                    } else {
                        preview
                    };
                    self.show_toast(format!("Copied \"{}\"", display_text));
                }
            }
            Message::CopyItem(id) => {
                if let Some(item) = self.items.iter().find(|i| i.id == id) {
                    if copy_item_to_clipboard(item).is_ok() {
                        self.show_toast("Copied item to clipboard!");
                    }
                }
            }
            Message::TogglePin(id) => {
                let _ = self.db.toggle_pin(&id);
                self.refresh_items();
            }
            Message::DeleteItem(id) => {
                let _ = self.db.delete_item(&id);
                self.items.retain(|i| i.id != id);
                self.show_toast("Item deleted");
            }
            Message::ClearUnpinned => {
                let _ = self.db.clear_unpinned();
                self.items.retain(|i| i.pinned);
                self.show_toast("Cleared unpinned history");
            }
            Message::Tick => {
                let mut new_received = false;
                while let Ok(item) = self.receiver.try_recv() {
                    let _ = self.db.save_item(&item);
                    self.items.retain(|i| i.id != item.id);
                    self.items.insert(0, item);
                    new_received = true;
                }
                if new_received {
                    let unpinned_count = self.items.iter().filter(|i| !i.pinned).count();
                    if unpinned_count > 100 {
                        let mut current_unpinned = 0;
                        self.items.retain(|i| {
                            if i.pinned {
                                true
                            } else {
                                current_unpinned += 1;
                                current_unpinned <= 100
                            }
                        });
                    }
                }
            }
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_millis(300)).map(|_| Message::Tick)
    }

    pub fn view(&self) -> Element<'_, Message> {
        // Sleek Window handle indicator
        let handle_bar = container(Space::with_height(0))
            .width(Length::Fixed(48.0))
            .height(Length::Fixed(4.0))
            .style(|_| container::Style {
                background: Some(COLOR_BORDER.into()),
                border: Border {
                    radius: 2.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

        let top_drag = container(handle_bar)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .padding(Padding { top: 8.0, right: 0.0, bottom: 4.0, left: 0.0 });

        // Navigation Tabs Header
        let tabs = [
            (ActiveTab::Clipboard, "Clipboard"),
            (ActiveTab::Favorites, "Pinned"),
            (ActiveTab::Emojis, "Emojis"),
            (ActiveTab::Symbols, "Symbols"),
        ];

        let mut tab_row = row![].spacing(6).align_y(alignment::Vertical::Center);
        for (tab, label) in tabs {
            let is_sel = self.active_tab == tab;
            let tab_btn = button(
                text(label)
                    .font(SYSTEM_FONT)
                    .size(13)
                    .color(if is_sel { Color::from_rgb(0.07, 0.07, 0.08) } else { COLOR_TEXT_SECONDARY }),
            )
            .padding([7.0, 14.0])
            .style(move |_, status| {
                let bg = if is_sel {
                    Color::from_rgb(0.89, 0.89, 0.91)
                } else if status == button::Status::Hovered {
                    COLOR_BG_CARD_HOVER
                } else {
                    COLOR_BG_CARD
                };
                button::Style {
                    background: Some(bg.into()),
                    border: Border {
                        radius: 8.0.into(),
                        width: 1.0,
                        color: if is_sel { Color::from_rgb(0.89, 0.89, 0.91) } else { COLOR_BORDER },
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
        .padding([4.0, 16.0])
        .width(Length::Fill);

        // Search Input Field
        let search_bar = text_input("Search history, emojis, symbols...", &self.search_query)
            .on_input(Message::SearchChanged)
            .padding([10.0, 14.0])
            .font(SYSTEM_FONT)
            .size(13)
            .style(|_, status| {
                let border_color = if status == text_input::Status::Focused {
                    COLOR_ACCENT
                } else {
                    COLOR_BORDER
                };
                text_input::Style {
                    background: COLOR_BG_CARD.into(),
                    border: Border {
                        radius: 8.0.into(),
                        width: 1.0,
                        color: border_color,
                    },
                    value: COLOR_TEXT_PRIMARY,
                    placeholder: COLOR_TEXT_MUTED,
                    selection: COLOR_ACCENT,
                    icon: COLOR_TEXT_MUTED,
                }
            });

        // Content View per Active Tab
        let content: Element<Message> = match self.active_tab {
            ActiveTab::Clipboard => self.view_clipboard_history(),
            ActiveTab::Favorites => self.view_favorites(),
            ActiveTab::Emojis => self.view_emojis(),
            ActiveTab::Symbols => self.view_symbols(),
        };

        let mut main_col = column![
            top_drag,
            Space::with_height(4),
            nav_bar,
            Space::with_height(8),
            container(search_bar).padding(Padding { top: 0.0, right: 16.0, bottom: 0.0, left: 16.0 }),
            Space::with_height(12),
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(Padding { top: 0.0, right: 16.0, bottom: 0.0, left: 16.0 })
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        // Toast Popup Notification at Bottom
        if let Some((msg, created_at)) = &self.toast_message {
            if created_at.elapsed() < Duration::from_secs(2) {
                let toast_box = container(
                    row![
                        text("✓")
                            .font(SYSTEM_FONT)
                            .size(13)
                            .color(Color::from_rgb(0.2, 0.85, 0.5)),
                        Space::with_width(8),
                        text(msg)
                            .font(SYSTEM_FONT)
                            .size(12)
                            .color(COLOR_TEXT_PRIMARY),
                    ]
                    .align_y(alignment::Vertical::Center)
                )
                .padding(Padding { top: 8.0, right: 16.0, bottom: 8.0, left: 16.0 })
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.06, 0.22, 0.16).into()),
                    border: Border {
                        radius: 8.0.into(),
                        width: 1.0,
                        color: Color::from_rgb(0.1, 0.6, 0.4),
                    },
                    shadow: iced::Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                        offset: Vector::new(0.0, 4.0),
                        blur_radius: 8.0,
                    },
                    ..Default::default()
                });
                main_col = main_col.push(
                    container(toast_box)
                        .padding(12.0)
                        .align_x(alignment::Horizontal::Center)
                        .width(Length::Fill)
                );
            }
        }

        container(main_col)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(COLOR_BG_APP.into()),
                ..Default::default()
            })
            .into()
    }

    fn view_clipboard_history(&self) -> Element<'_, Message> {
        let header_row = row![
            text("Clipboard History")
                .font(SYSTEM_FONT)
                .size(15)
                .color(COLOR_TEXT_PRIMARY),
            Space::with_width(Length::Fill),
            button(
                text("Clear unpinned")
                    .font(SYSTEM_FONT)
                    .size(12)
                    .color(COLOR_TEXT_SECONDARY)
            )
            .padding([6.0, 12.0])
            .style(|_, status| {
                let (bg, text_col, border_col) = if status == button::Status::Hovered {
                    (Color::from_rgb(0.25, 0.12, 0.14), COLOR_DANGER, COLOR_DANGER)
                } else {
                    (COLOR_BG_CARD, COLOR_TEXT_SECONDARY, COLOR_BORDER)
                };
                button::Style {
                    background: Some(bg.into()),
                    border: Border {
                        radius: 6.0.into(),
                        width: 1.0,
                        color: border_col,
                    },
                    text_color: text_col,
                    ..Default::default()
                }
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
                Space::with_height(60),
                text("Clipboard Empty")
                    .font(SYSTEM_FONT)
                    .size(16)
                    .color(COLOR_TEXT_SECONDARY),
                Space::with_height(6),
                text("Copied text and images will automatically appear here")
                    .font(SYSTEM_FONT)
                    .size(12)
                    .color(COLOR_TEXT_MUTED),
            ]
            .align_x(alignment::Horizontal::Center)
            .width(Length::Fill);

            return column![header_row, Space::with_height(12), empty_view].into();
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
                Space::with_height(60),
                text("No Pinned Items")
                    .font(SYSTEM_FONT)
                    .size(16)
                    .color(COLOR_TEXT_SECONDARY),
                Space::with_height(6),
                text("Click the Pin button on any item in Clipboard tab to save it here")
                    .font(SYSTEM_FONT)
                    .size(12)
                    .color(COLOR_TEXT_MUTED),
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
                let lines: Vec<&str> = t.lines().take(5).collect();
                let snippet = lines.join("\n");
                text(snippet)
                    .font(SYSTEM_FONT)
                    .size(13)
                    .color(COLOR_TEXT_PRIMARY)
                    .into()
            }
            ItemType::Image { width, height, png_bytes } => {
                let handle = image::Handle::from_bytes(png_bytes.clone());
                column![
                    image(handle).width(Length::Fixed(320.0)),
                    Space::with_height(4),
                    text(format!("Image ({} × {})", width, height))
                        .font(SYSTEM_FONT)
                        .size(11)
                        .color(COLOR_TEXT_MUTED)
                ]
                .into()
            }
        };

        let pin_label = if item.pinned { "Pinned" } else { "Pin" };
        let pin_text_color = if item.pinned {
            COLOR_PIN_ACTIVE
        } else {
            COLOR_TEXT_MUTED
        };

        let card_body = column![
            row![
                container(content_element).width(Length::Fill),
                button(text("Delete").font(SYSTEM_FONT).size(11).color(COLOR_TEXT_MUTED))
                    .padding([4.0, 8.0])
                    .style(|_, status| button::Style {
                        background: Some(
                            if status == button::Status::Hovered {
                                Color::from_rgb(0.25, 0.12, 0.14)
                            } else {
                                Color::TRANSPARENT
                            }
                            .into(),
                        ),
                        border: Border {
                            radius: 4.0.into(),
                            ..Default::default()
                        },
                        text_color: if status == button::Status::Hovered { COLOR_DANGER } else { COLOR_TEXT_MUTED },
                        ..Default::default()
                    })
                    .on_press(Message::DeleteItem(item.id.clone()))
            ],
            Space::with_height(8),
            row![
                Space::with_width(Length::Fill),
                button(
                    text(pin_label)
                        .font(SYSTEM_FONT)
                        .size(11)
                        .color(pin_text_color)
                )
                .padding([4.0, 8.0])
                .style(move |_, status| button::Style {
                    background: Some(
                        if status == button::Status::Hovered {
                            Color::from_rgb(0.2, 0.22, 0.3)
                        } else {
                            Color::TRANSPARENT
                        }
                        .into(),
                    ),
                    border: Border {
                        radius: 4.0.into(),
                        width: 1.0,
                        color: if item.pinned { COLOR_PIN_ACTIVE } else { COLOR_BORDER },
                    },
                    ..Default::default()
                })
                .on_press(Message::TogglePin(item.id.clone()))
            ]
        ];

        let id = item.id.clone();
        button(card_body)
            .width(Length::Fill)
            .padding(14.0)
            .style(|_, status| button::Style {
                background: Some(
                    if status == button::Status::Hovered {
                        COLOR_BG_CARD_HOVER
                    } else {
                        COLOR_BG_CARD
                    }
                    .into(),
                ),
                border: Border {
                    radius: 10.0.into(),
                    width: 1.0,
                    color: if status == button::Status::Hovered {
                        COLOR_BORDER_HOVER
                    } else {
                        COLOR_BORDER
                    },
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 6.0,
                },
                ..Default::default()
            })
            .on_press(Message::CopyItem(id))
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

        let mut cat_row = row![].spacing(6);
        for (cat, label) in categories {
            let is_sel = self.selected_emoji_cat == cat;
            let cat_btn = button(
                text(label)
                    .font(SYSTEM_FONT)
                    .size(11)
                    .color(if is_sel { Color::from_rgb(0.07, 0.07, 0.08) } else { COLOR_TEXT_MUTED }),
            )
            .padding([5.0, 10.0])
            .style(move |_, status| button::Style {
                background: Some(
                    if is_sel {
                        Color::from_rgb(0.89, 0.89, 0.91)
                    } else if status == button::Status::Hovered {
                        COLOR_BG_CARD_HOVER
                    } else {
                        COLOR_BG_CARD
                    }
                    .into(),
                ),
                border: Border {
                    radius: 6.0.into(),
                    width: 1.0,
                    color: if is_sel { Color::from_rgb(0.89, 0.89, 0.91) } else { COLOR_BORDER },
                },
                ..Default::default()
            })
            .on_press(Message::SelectEmojiCat(cat));

            cat_row = cat_row.push(cat_btn);
        }

        let query = self.search_query.trim().to_lowercase();
        let emojis: Vec<_> = get_emojis()
            .iter()
            .filter(|e| e.category == self.selected_emoji_cat)
            .filter(|e| query.is_empty() || e.name.contains(&query) || e.char.contains(&query))
            .collect();

        let mut grid_row = row![].spacing(8);
        let mut grid_col = column![].spacing(8);

        for (idx, e) in emojis.into_iter().enumerate() {
            let e_btn = button(text(e.char).font(EMOJI_FONT).size(22))
                .padding(8.0)
                .style(|_, status| button::Style {
                    background: Some(
                        if status == button::Status::Hovered {
                            COLOR_BG_CARD_HOVER
                        } else {
                            COLOR_BG_CARD
                        }
                        .into(),
                    ),
                    border: Border {
                        radius: 8.0.into(),
                        width: 1.0,
                        color: if status == button::Status::Hovered {
                            COLOR_BORDER_HOVER
                        } else {
                            COLOR_BORDER
                        },
                    },
                    ..Default::default()
                })
                .on_press(Message::CopyText(e.char.to_string()));

            grid_row = grid_row.push(e_btn);
            if (idx + 1) % 8 == 0 {
                grid_col = grid_col.push(grid_row);
                grid_row = row![].spacing(8);
            }
        }
        grid_col = grid_col.push(grid_row);

        column![
            scrollable(cat_row).direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::default())),
            Space::with_height(10),
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

        let mut cat_row = row![].spacing(6);
        for (cat, label) in categories {
            let is_sel = self.selected_symbol_cat == cat;
            let cat_btn = button(
                text(label)
                    .font(SYSTEM_FONT)
                    .size(11)
                    .color(if is_sel { Color::from_rgb(0.07, 0.07, 0.08) } else { COLOR_TEXT_MUTED }),
            )
            .padding([5.0, 10.0])
            .style(move |_, status| button::Style {
                background: Some(
                    if is_sel {
                        Color::from_rgb(0.89, 0.89, 0.91)
                    } else if status == button::Status::Hovered {
                        COLOR_BG_CARD_HOVER
                    } else {
                        COLOR_BG_CARD
                    }
                    .into(),
                ),
                border: Border {
                    radius: 6.0.into(),
                    width: 1.0,
                    color: if is_sel { Color::from_rgb(0.89, 0.89, 0.91) } else { COLOR_BORDER },
                },
                ..Default::default()
            })
            .on_press(Message::SelectSymbolCat(cat));

            cat_row = cat_row.push(cat_btn);
        }

        let query = self.search_query.trim().to_lowercase();
        let symbols: Vec<_> = get_symbols()
            .iter()
            .filter(|s| s.category == self.selected_symbol_cat)
            .filter(|s| query.is_empty() || s.name.contains(&query) || s.symbol.contains(&query))
            .collect();

        let mut grid_row = row![].spacing(8);
        let mut grid_col = column![].spacing(8);

        for (idx, sym) in symbols.into_iter().enumerate() {
            let sym_btn = button(
                text(sym.symbol)
                    .font(SYSTEM_FONT)
                    .size(18)
                    .color(COLOR_TEXT_PRIMARY)
            )
            .padding(10.0)
            .style(|_, status| button::Style {
                background: Some(
                    if status == button::Status::Hovered {
                        COLOR_BG_CARD_HOVER
                    } else {
                        COLOR_BG_CARD
                    }
                    .into(),
                ),
                border: Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: if status == button::Status::Hovered {
                        COLOR_BORDER_HOVER
                    } else {
                        COLOR_BORDER
                    },
                },
                ..Default::default()
            })
            .on_press(Message::CopyText(sym.symbol.to_string()));

            grid_row = grid_row.push(sym_btn);
            if (idx + 1) % 8 == 0 {
                grid_col = grid_col.push(grid_row);
                grid_row = row![].spacing(8);
            }
        }
        grid_col = grid_col.push(grid_row);

        column![
            scrollable(cat_row).direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::default())),
            Space::with_height(10),
            scrollable(grid_col).height(Length::Fill)
        ]
        .into()
    }
}
