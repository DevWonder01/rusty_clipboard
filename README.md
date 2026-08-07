# Rusty Clipboard

A high-performance Windows 11 Clipboard replica built in Rust using Iced for the user interface and heed for local key-value storage.

## Features

- **Clipboard History**: Automatically monitors and saves copied text snippets and raw RGBA images from the system clipboard.
- **Embedded Database (`heed`)**: Uses LMDB via the `heed` crate for fast, zero-copy, transactional local storage.
- **Pinned Items**: Pin important text or images to prevent them from being removed when clearing clipboard history.
- **Emoji Picker**: Searchable catalog of emojis grouped by categories (Smileys, Animals, Food, Sports, Travel, Objects, Symbols).
- **Symbol Picker**: Quick access to currency symbols, mathematical operators, punctuation marks, arrows, and Greek letters.
- **Kaomoji & ASCII Expressions**: Collection of Japanese kaomoji emoticons ready to copy with a single click.
- **Search & Filter**: Real-time filtering across clipboard history, emojis, and symbols.
- **Fluent UI Aesthetic**: Custom styling matching the Windows 11 clipboard modal design using Iced widgets.

## Architecture

The project consists of the following core modules:

- `src/main.rs`: Entry point initializing the Iced application window, database, and background listener thread.
- `src/db.rs`: Database layer wrapping `heed::Env` and `heed::Database` for storing serialized clipboard items.
- `src/clipboard_listener.rs`: Background worker polling system clipboard changes using `arboard`.
- `src/app.rs`: State management, event handling, and view layout implemented using Iced.
- `src/catalog.rs`: Static catalogs for emojis, symbols, and kaomoji expressions.
- `src/types.rs`: Core data types and data models.

## Dependencies

- **iced**: Cross-platform GUI library for Rust focused on simplicity and type safety.
- **heed**: High-level, type-safe Rust binding to LMDB.
- **arboard**: Cross-platform system clipboard access.
- **serde / serde_json**: Serialization and deserialization of clipboard items.
- **image**: Image processing library for thumbnail rendering.

## Building and Running

### Prerequisites

Ensure you have Rust and Cargo installed on your system. On Linux systems, X11 or Wayland clipboard development libraries may be required by `arboard`:

```bash
sudo apt-get install libx11-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

### Note on Ubuntu

Ubuntu (GNOME) does not provide a clipboard history UI by default, which can be frustrating when coming from Windows. If you plan to use this app on Ubuntu, you may want to install a clipboard manager such as `copyq` to get history, pinning, and more reliable image/text handling:

```bash
sudo apt-get install copyq
```

Clipboard manager tools provide history, pinning, and improved clipboard behavior that complement this app on Linux.

### Run in Development Mode

```bash
cargo run
```

### Build Release Binary

```bash
cargo build --release
```

The compiled executable will be located at `target/release/rusty_clipboard`.

## License

MIT License.
