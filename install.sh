#!/bin/bash
set -e

REPO="benfaerber/umlauter-rs"
BIN_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/umlauter"
APPS_DIR="$HOME/.local/share/applications"
AUTOSTART_DIR="$HOME/.config/autostart"
TMP_DIR=$(mktemp -d)

cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

info() { echo -e "\033[1;34m::\033[0m $1"; }
warn() { echo -e "\033[1;33m::\033[0m $1"; }
err() { echo -e "\033[1;31m::\033[0m $1"; exit 1; }

# Check dependencies
command -v curl >/dev/null || err "curl is required"
command -v cargo >/dev/null || err "rust/cargo is required (https://rustup.rs)"
command -v xdotool >/dev/null || warn "xdotool not found - install it: sudo apt install xdotool"
command -v dbus-daemon >/dev/null || warn "dbus not found - tray icon won't work"

info "Cloning $REPO..."
curl -sL "https://github.com/$REPO/archive/refs/heads/master.tar.gz" | tar xz -C "$TMP_DIR"
SRC_DIR="$TMP_DIR/$(ls "$TMP_DIR")"

info "Building..."
cargo build --release --manifest-path "$SRC_DIR/Cargo.toml" --quiet

info "Installing binary to $BIN_DIR..."
mkdir -p "$BIN_DIR"
cp "$SRC_DIR/target/release/umlauter" "$BIN_DIR/umlauter"
chmod +x "$BIN_DIR/umlauter"

if [ ! -f "$CONFIG_DIR/umlauter.toml" ]; then
    info "Installing config to $CONFIG_DIR..."
    mkdir -p "$CONFIG_DIR"
    cp "$SRC_DIR/umlauter.toml" "$CONFIG_DIR/umlauter.toml"
else
    info "Config already exists at $CONFIG_DIR/umlauter.toml, skipping"
fi

info "Installing desktop entry..."
mkdir -p "$APPS_DIR"
cat > "$APPS_DIR/umlauter.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Umlauter
Comment=Type accented characters with keyboard shortcuts
Exec=$BIN_DIR/umlauter
Icon=input-keyboard
Terminal=false
Categories=Utility;
StartupNotify=false
X-GNOME-Autostart-enabled=true
EOF
update-desktop-database "$APPS_DIR" 2>/dev/null || true

info "Enabling autostart..."
mkdir -p "$AUTOSTART_DIR"
ln -sf "$APPS_DIR/umlauter.desktop" "$AUTOSTART_DIR/umlauter.desktop"

# Permissions
NEEDS_LOGOUT=false

if ! groups | grep -q '\binput\b'; then
    info "Adding $USER to input group..."
    sudo usermod -aG input "$USER"
    NEEDS_LOGOUT=true
fi

if [ ! -f /etc/udev/rules.d/99-uinput.rules ]; then
    info "Creating udev rule for /dev/uinput..."
    echo 'KERNEL=="uinput", GROUP="input", MODE="0660"' | sudo tee /etc/udev/rules.d/99-uinput.rules >/dev/null
    sudo udevadm control --reload-rules && sudo udevadm trigger
fi

echo ""
info "Umlauter installed!"
echo ""
echo "  Binary:    $BIN_DIR/umlauter"
echo "  Config:    $CONFIG_DIR/umlauter.toml"
echo "  Desktop:   $APPS_DIR/umlauter.desktop"
echo "  Autostart: $AUTOSTART_DIR/umlauter.desktop"
echo ""

if [ "$NEEDS_LOGOUT" = true ]; then
    warn "Log out and back in for input group permissions to take effect."
    echo ""
fi

if echo "$PATH" | grep -q "$BIN_DIR"; then
    info "Run 'umlauter' or launch it from your application menu."
else
    warn "$BIN_DIR is not in your PATH."
    info "Run '$BIN_DIR/umlauter' or add $BIN_DIR to your PATH."
fi
