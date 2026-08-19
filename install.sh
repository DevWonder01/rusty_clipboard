#!/usr/bin/env bash
set -e

# Copy screenshot to assets if available
SCREENSHOT_SRC="$HOME/.gemini/antigravity/brain/a4ef319b-fb3f-498a-8dac-059ad02cb3be/media__1787125754805.png"
if [ -f "$SCREENSHOT_SRC" ]; then
    cp "$SCREENSHOT_SRC" assets/screenshot.png 2>/dev/null || true
fi

echo "🚀 Building Rusty Clipboard in release mode..."
cargo build --release

BIN_DIR="$HOME/.local/bin"
APP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"

mkdir -p "$BIN_DIR"
mkdir -p "$APP_DIR"
mkdir -p "$ICON_DIR"

echo "📦 Installing binary to $BIN_DIR/rusty_clipboard..."
cp target/release/rusty_clipboard "$BIN_DIR/rusty_clipboard"
chmod +x "$BIN_DIR/rusty_clipboard"

echo "🎨 Installing application icon to $ICON_DIR/rusty-clipboard.svg..."
cp assets/rusty-clipboard.svg "$ICON_DIR/rusty-clipboard.svg"

echo "🖥️  Installing desktop shortcut to $APP_DIR/rusty-clipboard.desktop..."
cp rusty-clipboard.desktop "$APP_DIR/rusty-clipboard.desktop"

# Refresh desktop database & icon cache if tools exist
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APP_DIR" 2>/dev/null || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
fi

echo "✅ Installation complete!"
echo "You can now open 'Rusty Clipboard' directly from your Linux app menu/launcher."
