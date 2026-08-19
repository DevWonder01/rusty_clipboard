#!/usr/bin/env bash
set -e

BIN_DIR="$HOME/.local/bin"
APP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"

echo "🗑️  Removing Rusty Clipboard..."
rm -f "$BIN_DIR/rusty_clipboard"
rm -f "$APP_DIR/rusty-clipboard.desktop"
rm -f "$ICON_DIR/rusty-clipboard.svg"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APP_DIR" 2>/dev/null || true
fi

echo "✅ Rusty Clipboard has been uninstalled."
