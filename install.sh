#!/usr/bin/env bash
set -e

# Generate PNG icon asset if needed
if command -v python3 >/dev/null 2>&1; then
    python3 assets/generate_png.py 2>/dev/null || true
fi

echo "Building Rusty Clipboard in release mode..."
cargo build --release

BIN_DIR="$HOME/.local/bin"
APP_DIR="$HOME/.local/share/applications"
ICON_SCALABLE="$HOME/.local/share/icons/hicolor/scalable/apps"
ICON_128="$HOME/.local/share/icons/hicolor/128x128/apps"
ICON_48="$HOME/.local/share/icons/hicolor/48x48/apps"
PIXMAPS_DIR="$HOME/.local/share/pixmaps"

mkdir -p "$BIN_DIR"
mkdir -p "$APP_DIR"
mkdir -p "$ICON_SCALABLE"
mkdir -p "$ICON_128"
mkdir -p "$ICON_48"
mkdir -p "$PIXMAPS_DIR"

echo "Installing binary to $BIN_DIR/rusty_clipboard..."
# Remove old binary first if running to avoid text file busy error
rm -f "$BIN_DIR/rusty_clipboard"
cp target/release/rusty_clipboard "$BIN_DIR/rusty_clipboard"
chmod +x "$BIN_DIR/rusty_clipboard"

echo "Installing application icons..."
if [ -f assets/rusty-clipboard.svg ]; then
    cp assets/rusty-clipboard.svg "$ICON_SCALABLE/rusty-clipboard.svg"
fi

if [ -f assets/rusty-clipboard.png ]; then
    cp assets/rusty-clipboard.png "$ICON_128/rusty-clipboard.png"
    cp assets/rusty-clipboard.png "$ICON_48/rusty-clipboard.png"
    cp assets/rusty-clipboard.png "$PIXMAPS_DIR/rusty-clipboard.png"
fi

echo "Installing desktop shortcut to $APP_DIR/rusty-clipboard.desktop..."
cp rusty-clipboard.desktop "$APP_DIR/rusty-clipboard.desktop"

# Refresh desktop database & icon cache if tools exist
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APP_DIR" 2>/dev/null || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
fi

echo "Installation complete!"
echo "You can now open 'Rusty Clipboard' directly from your Linux app menu/launcher."
