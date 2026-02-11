# Umlauter

A lightweight Linux tray application that lets you type accented characters using simple letter sequences. Type `aee` and it becomes `ä`, type `oee` and it becomes `ö`, etc.

## How it works

Umlauter monitors your keyboard via evdev. When it detects a trigger sequence, it erases the typed characters with backspaces and inserts the replacement using xdotool.

| You type                  | You get |
|---------------------------|---------|
| `aee`                     | ä       |
| `oee`                     | ö       |
| `uee`                     | ü       |
| `sse`                     | ß       |
| `Aee` / `AEE` / `AeE`... | Ä       |
| `Oee` / `OEE` / `OeE`... | Ö       |
| `Uee` / `UEE` / `UeE`... | Ü       |

Matching is case-insensitive. If the first letter is uppercase, the replacement is uppercase.

All mappings are configurable in `umlauter.toml`.

## Quick Install

```bash
curl -sL https://raw.githubusercontent.com/benfaerber/umlauter-rs/master/install.sh | bash
```

This builds from source, installs the binary, config, desktop entry, sets up autostart, and configures permissions. You may be prompted for your sudo password.

## Requirements

- Linux with X11
- `xdotool` (`sudo apt install xdotool`)
- Access to `/dev/input` and `/dev/uinput` (see Setup)

## Build

```bash
cargo build --release
```

## Setup

Your user needs permission to read keyboard input and write to uinput. Run the setup script or do it manually:

```bash
# Add yourself to the input group
sudo usermod -aG input $USER

# Allow the input group to write to /dev/uinput
echo 'KERNEL=="uinput", GROUP="input", MODE="0660"' | sudo tee /etc/udev/rules.d/99-uinput.rules
sudo udevadm control --reload-rules && sudo udevadm trigger
```

**Log out and back in** for the group change to take effect.

## Manual Install

If you prefer to do it yourself after building:

```bash
cp target/release/umlauter ~/.local/bin/
mkdir -p ~/.config/umlauter
cp umlauter.toml ~/.config/umlauter/
cp umlauter.desktop ~/.local/share/applications/

# Optional: autostart on login
mkdir -p ~/.config/autostart
ln -s ~/.local/share/applications/umlauter.desktop ~/.config/autostart/
```

## Usage

Launch from your application menu or run directly:

```bash
umlauter
```

A tray icon appears with options to pause/resume or quit. The config file is loaded from the first path found:

1. `./umlauter.toml` (current directory)
2. `~/.config/umlauter/umlauter.toml`

## Configuration

Edit `~/.config/umlauter/umlauter.toml` to add or change mappings:

```toml
[mappings]
aee = "ä"
oee = "ö"
uee = "ü"
```

Each key is the trigger sequence you type, and the value is the character that replaces it.
