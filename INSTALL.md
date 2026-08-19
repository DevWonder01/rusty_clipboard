# Installation Guide for Rusty Clipboard

This guide provides step-by-step instructions for compiling, installing, and running **Rusty Clipboard** as a native desktop application on Linux.

---

## System Prerequisites

Before installing, ensure your system has the Rust toolchain installed alongside the required clipboard libraries for your Linux distribution.

### 1. Install Rust Toolchain
If Rust is not already installed, install it via `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Install Linux System Dependencies

#### Debian / Ubuntu / Pop!_OS / Mint:
```bash
sudo apt-get update
sudo apt-get install -y build-essential libx11-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

#### Fedora / RHEL:
```bash
sudo dnf install -y gcc libX11-devel libxcb-devel
```

#### Arch Linux / Manjaro:
```bash
sudo pacman -S --needed base-devel libx11 libxcb
```

---

## Quick Automated Installation

The easiest way to install Rusty Clipboard is using the provided installation script:

```bash
chmod +x install.sh
./install.sh
```

### What `install.sh` does automatically:
1. Compiles the optimized release binary (`cargo build --release`).
2. Copies the executable to `~/.local/bin/rusty_clipboard`.
3. Installs the application icon to `~/.local/share/icons/hicolor/scalable/apps/rusty-clipboard.svg`.
4. Installs the desktop launcher entry to `~/.local/share/applications/rusty-clipboard.desktop`.
5. Updates your system desktop application database and icon cache.

---

## Manual Installation (Step-by-Step)

If you prefer to perform the installation manually without running the script:

### Step 1: Build the Release Binary
```bash
cargo build --release
```

### Step 2: Create Local Application Directories
```bash
mkdir -p ~/.local/bin
mkdir -p ~/.local/share/applications
mkdir -p ~/.local/share/icons/hicolor/scalable/apps
```

### Step 3: Copy Binary, Icon, and Desktop Launcher
```bash
# Install binary
cp target/release/rusty_clipboard ~/.local/bin/rusty_clipboard
chmod +x ~/.local/bin/rusty_clipboard

# Install SVG Icon
cp assets/rusty-clipboard.svg ~/.local/share/icons/hicolor/scalable/apps/rusty-clipboard.svg

# Install Desktop Launcher
cp rusty-clipboard.desktop ~/.local/share/applications/rusty-clipboard.desktop
```

### Step 4: Refresh Desktop Application Database
```bash
update-desktop-database ~/.local/share/applications 2>/dev/null || true
gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor 2>/dev/null || true
```

---

## Launching Rusty Clipboard

Once installed, you can launch Rusty Clipboard in two ways:

1. **From your System Application Launcher**:
   Press Super / Windows Key and search for **"Rusty Clipboard"**.
2. **From Terminal**:
   Run `rusty_clipboard` (Make sure `~/.local/bin` is in your `$PATH`).

---

## Autostart on Linux Login (Optional)

To automatically launch Rusty Clipboard every time you log into your Linux desktop session:

```bash
mkdir -p ~/.config/autostart
cp ~/.local/share/applications/rusty-clipboard.desktop ~/.config/autostart/
```

---

## Uninstallation

To completely remove Rusty Clipboard from your system, run:

```bash
chmod +x uninstall.sh
./uninstall.sh
```

Or remove the installed files manually:
```bash
rm -f ~/.local/bin/rusty_clipboard
rm -f ~/.local/share/applications/rusty-clipboard.desktop
rm -f ~/.local/share/icons/hicolor/scalable/apps/rusty-clipboard.svg
rm -f ~/.config/autostart/rusty-clipboard.desktop
```
