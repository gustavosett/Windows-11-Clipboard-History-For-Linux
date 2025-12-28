<img width="897" height="427" alt="image" src="https://github.com/user-attachments/assets/74400c8b-9d7d-49ce-8de7-45dfd556e256" />

<div align="center">

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.77+-orange.svg)
![Tauri](https://img.shields.io/badge/tauri-v2-blue.svg)
![Platform](https://img.shields.io/badge/platform-linux-lightgrey.svg)
![Version](https://img.shields.io/github/v/release/gustavosett/Windows-11-Clipboard-History-For-Linux?color=green)
![Sites](https://img.shields.io/website?down_color=red&down_message=offline&up_color=green&up_message=online&url=https%3A%2F%2Fclipboard.gustavosett.dev)

**A beautiful, [Windows 11-style Clipboard History Manager for Linux](https://clipboard.gustavosett.dev).**

*Works on Wayland & X11.*

Built with 🦀 **Rust** + ⚡ **Tauri v2** + ⚛️ **React** + 🎨 **Tailwind CSS**

[Features](#-features) • [Installation](#-installation) • [How to Use](#-how-to-use) • [Development](#-development)

</div>

---

## ✨ Features

- 🐧 **Wayland & X11 Support** - Uses OS-level shortcuts and `uinput` for pasting to support Wayland & X11.
- ⚡ **Global Hotkey** - Press `Super+V` or `Ctrl+Alt+V` to open instantly.
- 🖱️ **Smart Positioning** - Window follows your mouse cursor across multiple monitors.
- 📌 **Pinning** - Keep important items at the top of your list.
- 🖼️ **Rich Media** - Supports Images, Text, etc.
- 🎬 **GIF Integration** - Search and paste GIFs from Tenor directly into Discord, Slack, etc.
- 🤩 **Emoji Picker** - Built-in searchable emoji keyboard.
- 🏎️ **Performance** - Native Rust backend ensures minimal resource usage.
- 🛡️ **Privacy Focused** - History is stored locally and never leaves your machine.
- 🧙 **Setup Wizard** - First-run wizard guides you through permission setup, detects shortcut conflicts, and autostart configuration.

---

## 📥 Installation

### 🚀 Recommended: One-Line Install

This script automatically detects your distro and architecture (x86_64, ARM64), downloads the correct package, and sets up permissions.

```bash
curl -fsSL https://raw.githubusercontent.com/gustavosett/Windows-11-Clipboard-History-For-Linux/master/scripts/install.sh | bash
```

> **Note:** The installer uses ACLs to grant immediate access to input devices — **no logout required!**

### 📦 Manual Installation

Download the latest release from the [Releases Page](https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux/releases).

<details>
<summary><b>Debian / Ubuntu / Pop!_OS / Linux Mint</b></summary>

```bash
# Download and install (replace VERSION with actual version)
sudo apt install ./win11-clipboard-history_VERSION_amd64.deb

# The package sets up udev rules automatically.
# You may need to log out and back in for permissions to take effect,
# or run this for immediate access:
sudo setfacl -m u:$USER:rw /dev/uinput
```

</details>

<details>
<summary><b>Fedora / RHEL / CentOS</b></summary>

```bash
# Download and install (replace VERSION with actual version)
sudo dnf install ./win11-clipboard-history-VERSION-1.x86_64.rpm

# For immediate access:
sudo setfacl -m u:$USER:rw /dev/uinput
```

</details>

<details>
<summary><b>Arch Linux (AUR)</b></summary>

```bash
# Using yay
yay -S win11-clipboard-history-bin

# Or using paru
paru -S win11-clipboard-history-bin
```

</details>

<details>
<summary><b>AppImage (Universal)</b></summary>

```bash
# Download the AppImage
chmod +x win11-clipboard-history_*.AppImage

# Run it
./win11-clipboard-history_*.AppImage

# For paste to work, grant uinput access:
sudo setfacl -m u:$USER:rw /dev/uinput
```

> **Note:** AppImage is fully portable — no system installation required. The permission command above is only needed for paste simulation.

</details>

<details>
<summary><b>Build from Source</b></summary>

```bash
# Clone and enter the repo
git clone https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux.git
cd Windows-11-Clipboard-History-For-Linux

# Install dependencies (auto-detects distro)
make deps
make rust
make node
source ~/.cargo/env

# Build
make build

# Install system-wide (uses /usr/local by default)
sudo make install

# Or install to /usr like a package
sudo make install PREFIX=/usr
```

</details>

### 🎯 First Run

On the first launch, the app will show a **Setup Wizard** that:
- ✅ Checks if you have the necessary permissions for paste simulation
- 🔧 Offers a one-click fix if permissions are missing
- ⚠️ **Detects shortcut conflicts** with your desktop environment (GNOME, KDE, i3, Sway, Hyprland, etc.)
- ⚡ Offers automatic conflict resolution where possible
- ⌨️ Helps register the global shortcut (Super+V) for your desktop environment
- 🚀 Lets you enable autostart on login

---

## ⌨️ How to Use

| Hotkey | Action |
| :--- | :--- |
| **`Super + V`** | Open Clipboard History |
| **`Ctrl + Alt + V`** | Alternative Shortcut |
| **`Esc`** | Close Window |
| **`↑ / ↓ / Tab`** | Navigate Items |
| **`Enter`** | Paste Selected Item |

### Tips
- **Paste GIFs:** Select a GIF, and it will be copied as a file URI. The app simulates `Ctrl+V` to paste it into apps like Discord or Telegram.
- **Pinning:** Click the pin icon on any item to keep it at the top permanently.

---

## 🛠️ Development

### Prerequisites

- **Rust 1.77+**
- **Node.js 20+**
- System build dependencies (see `make deps`)

### Quick Start

```bash
git clone https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux.git
cd Windows-11-Clipboard-History-For-Linux

make deps      # Install system dependencies (auto-detects distro)
make rust      # Install Rust via rustup
make node      # Install Node.js via nvm
source ~/.cargo/env

make dev       # Run in development mode with hot reload
```

### Available Commands

| Command | Description |
|---------|-------------|
| `make dev` | Run in development mode |
| `make build` | Build production release |
| `make install` | Install to system (default: `/usr/local`) |
| `make uninstall` | Remove from system |
| `make clean` | Remove build artifacts |
| `make lint` | Run linters |
| `make help` | Show all available commands |

---

## 🔧 Troubleshooting

### App won't open with Super+V

1. **Ensure the app is running:** `pgrep -f win11-clipboard-history-bin`
2. If not running, launch it from your app menu or run `win11-clipboard-history`
3. **Re-run the Setup Wizard** to register the shortcut:
   ```bash
   rm ~/.config/win11-clipboard-history/setup.json
   win11-clipboard-history
   ```

### Super+V Conflicts with Desktop Environment

Many desktop environments use Super+V for built-in features. The Setup Wizard will detect and offer to fix these automatically, but you can also resolve them manually:

<details>
<summary><b>GNOME / Ubuntu</b></summary>

GNOME uses Super+V for the Notification Center / Message Tray.

```bash
# Change GNOME's notification tray shortcut to Super+Shift+V
gsettings set org.gnome.shell.keybindings toggle-message-tray "['<Super><Shift>v']"
```

Or manually: **Settings → Keyboard → Keyboard Shortcuts → Search "Notification"**

</details>

<details>
<summary><b>Pop!_OS / Pop Shell</b></summary>

Pop!_OS inherits GNOME's Super+V shortcut:

```bash
gsettings set org.gnome.shell.keybindings toggle-message-tray "['<Super><Shift>v']"
```

If Pop Shell also uses Super+V for tiling:
**Settings → Keyboard → Customize Shortcuts → Pop Shell**

</details>

<details>
<summary><b>KDE Plasma</b></summary>

Check if Klipper (built-in clipboard manager) uses Meta+V:
1. Right-click Klipper in system tray → Configure
2. Go to Shortcuts
3. Change or disable the Meta+V binding

Or: **System Settings → Shortcuts → Global Shortcuts → Search "Meta+V"**

</details>

<details>
<summary><b>COSMIC Desktop</b></summary>

**Settings → Keyboard → Shortcuts** and check for Super+V bindings in both Custom and System shortcuts.

</details>

<details>
<summary><b>i3 Window Manager</b></summary>

Edit your i3 config (`~/.config/i3/config`):

```bash
# Comment out or remove existing $mod+v binding
# bindsym $mod+v split vertical

# Add clipboard history
bindsym $mod+v exec win11-clipboard-history
```

Reload i3: `$mod+Shift+r`

</details>

<details>
<summary><b>Sway</b></summary>

Edit your Sway config (`~/.config/sway/config`):

```bash
# Comment out existing $mod+v binding if any
# Add clipboard history
bindsym $mod+v exec win11-clipboard-history
```

Reload Sway: `$mod+Shift+c`

</details>

<details>
<summary><b>Hyprland</b></summary>

Edit your Hyprland config (`~/.config/hypr/hyprland.conf`):

```bash
# Comment out existing SUPER, V binding if any
# Add clipboard history
bind = SUPER, V, exec, win11-clipboard-history
```

Config auto-reloads.

</details>

### Pasting doesn't work

1. **Check the Setup Wizard:** It shows permission status and offers one-click fixes
2. **Quick fix:** `sudo setfacl -m u:$USER:rw /dev/uinput`
3. **Wayland:** Ensure `wl-clipboard` is installed
4. **X11:** Ensure `xclip` is installed
5. The app simulates `Ctrl+V` — ensure the target app accepts this shortcut

### Window appears on the wrong monitor
The app uses smart cursor tracking. If it appears incorrectly, try moving your mouse to the center of the desired screen and pressing the hotkey again.

---

## 🗑️ Uninstalling

<details>
<summary><b>Debian / Ubuntu</b></summary>

```bash
sudo apt remove win11-clipboard-history
# To also remove config files:
sudo apt purge win11-clipboard-history
```

</details>

<details>
<summary><b>Fedora / RHEL</b></summary>

```bash
sudo dnf remove win11-clipboard-history
```

</details>

<details>
<summary><b>Arch Linux (AUR)</b></summary>

```bash
yay -R win11-clipboard-history-bin
```

</details>

<details>
<summary><b>AppImage</b></summary>

```bash
rm -f ~/.local/bin/win11-clipboard-history*
rm -f ~/.local/share/applications/win11-clipboard-history.desktop
rm -rf ~/.config/win11-clipboard-history
```

</details>

<details>
<summary><b>Built from Source (Makefile)</b></summary>

```bash
rm -f ~/.local/bin/win11-clipboard-history
rm -rf ~/.local/lib/win11-clipboard-history
rm -f ~/.config/autostart/win11-clipboard-history.desktop
```


**Check if it still have shortcuts registered and remove them:**
> This can happen if the application was uninstalled while it was running or if the uninstall permissions were incorrect.

1. Go to Settings -> Keyboard -> Shortcuts
2. Find "Win11 Clipboard History" or similar entry
3. Remove the shortcut or change it to "Disabled"

</details>

---

![Screenshot](./docs/img/banner.gif)

## Contributors ✨

Thanks goes to these wonderful people ([emoji key](https://allcontributors.org/docs/en/emoji-key)):

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/freshCoder21313"><img src="https://avatars.githubusercontent.com/u/151538542?v=4?s=100" width="100px;" alt="freshCoder21313"/><br /><sub><b>freshCoder21313</b></sub></a><br /><a href="#data-freshCoder21313" title="Data">🔣</a></td>
    </tr>
  </tbody>
  <tfoot>
    <tr>
      <td align="center" size="13px" colspan="7">
        <img src="https://raw.githubusercontent.com/all-contributors/all-contributors-cli/1b8533af435da9854653492b1327a23a4dbd0a10/assets/logo-small.svg">
          <a href="https://all-contributors.js.org/docs/en/bot/usage">Add your contributions</a>
        </img>
      </td>
    </tr>
  </tfoot>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind welcome!

## 🤝 Contributing

Contributions are welcome!
1. Fork it
2. Create your feature branch (`git checkout -b feature/cool-feature`)
3. Commit your changes (`git commit -m 'feat: add cool feature'`)
4. Push to the branch (`git push origin feature/cool-feature`)
5. Open a Pull Request

## 📄 License

MIT License © [Gustavo Sett](https://github.com/gustavosett)

<div align="center">
  <br />
  <b>If you like this project, give it a ⭐!</b>
</div>
