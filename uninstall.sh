#!/usr/bin/env bash
set -e

BIN_DIR="$HOME/.local/bin"
APP_DIR="$HOME/.local/share/applications"
ICON_SCALABLE="$HOME/.local/share/icons/hicolor/scalable/apps"
ICON_128="$HOME/.local/share/icons/hicolor/128x128/apps"
ICON_48="$HOME/.local/share/icons/hicolor/48x48/apps"
PIXMAPS_DIR="$HOME/.local/share/pixmaps"

echo "Removing Rusty Clipboard..."
rm -f "$BIN_DIR/rusty_clipboard"
rm -f "$APP_DIR/rusty-clipboard.desktop"
rm -f "$ICON_SCALABLE/rusty-clipboard.svg"
rm -f "$ICON_128/rusty-clipboard.png"
rm -f "$ICON_48/rusty-clipboard.png"
rm -f "$PIXMAPS_DIR/rusty-clipboard.png"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APP_DIR" 2>/dev/null || true
fi

echo "Rusty Clipboard has been uninstalled."
