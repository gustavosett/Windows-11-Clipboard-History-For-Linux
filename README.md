# 📋 Windows 11 Clipboard History For Linux

<div align="center">

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![Tauri](https://img.shields.io/badge/tauri-v2-blue.svg)
![Platform](https://img.shields.io/badge/platform-linux-lightgrey.svg)
![Version](https://img.shields.io/github/v/release/gustavosett/Windows-11-Clipboard-History-For-Linux?color=green)

**A beautiful, Windows 11-style Clipboard History Manager for Linux.**  
*Works on Wayland & X11.*

![Screenshot](./docs/img/win11-clipboard-history.png)
![Screenshot](./docs/img/dynamic_themes.jpg)

Built with 🦀 **Rust** + ⚡ **Tauri v2** + ⚛️ **React** + 🎨 **Tailwind CSS**

[Features](#-features) • [Installation](#-installation) • [How to Use](#-how-to-use) • [Development](#-development)

</div>

---

## ✨ Features

- 🐧 **Wayland & X11 Support** - Uses kernel-level input handling (`evdev` & `uinput`) to bypass Wayland restrictions.
- ⚡ **Global Hotkey** - Press `Super+V` or `Ctrl+Alt+V` to open instantly.
- 🖱️ **Smart Positioning** - Window follows your mouse cursor across multiple monitors.
- 📌 **Pinning** - Keep important items at the top of your list.
- 🖼️ **Rich Media** - Supports Images, Text, etc.
- 🎬 **GIF Integration** - Search and paste GIFs from Tenor directly into Discord, Slack, etc.
- 🤩 **Emoji Picker** - Built-in searchable emoji keyboard.
- 🏎️ **Performance** - Native Rust backend ensures minimal resource usage.
- 🛡️ **Privacy Focused** - History is stored locally and never leaves your machine.

---

## 📥 Installation

### 🚀 Recommended: One-Line Install

This script automatically detects your distro, downloads the correct package (DEB, RPM, or AppImage), sets up permissions, and configures autostart.

```bash
curl -sL http://install-clipboard.gustavosett.dev | bash
```

> **Note:** This installer uses ACLs to grant immediate access to input devices, so **no logout is required**!

### 📦 Manual Installation

If you prefer to install manually, download the latest release from the [Releases Page](https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux/releases).

<details>
<summary><b>Click to see manual setup instructions (for other formats or troubleshooting)</b></summary>

1. **Install the package:**
   - **Debian/Ubuntu:** `sudo apt install ./win11-clipboard-history_*.deb`
   - **Fedora/RHEL:** `sudo dnf install ./win11-clipboard-history-*.rpm`
   - **AppImage:** Make executable (`chmod +x`) and run.

2. **Configure Permissions (Crucial):**
   Since this app listens to global hotkeys on Wayland, it needs access to `/dev/input`.

   ```bash
   # 1. Create 'input' group
   sudo groupadd input
   sudo usermod -aG input $USER

   # 2. Create udev rules
   echo 'KERNEL=="event*", SUBSYSTEM=="input", MODE="0660", GROUP="input"' | sudo tee /etc/udev/rules.d/99-win11-clipboard-input.rules
   echo 'KERNEL=="uinput", SUBSYSTEM=="misc", MODE="0660", GROUP="input", OPTIONS+="static_node=uinput"' | sudo tee -a /etc/udev/rules.d/99-win11-clipboard-input.rules

   # 3. Load uinput module
   sudo modprobe uinput
   echo "uinput" | sudo tee /etc/modules-load.d/uinput.conf

   # 4. Reload rules
   sudo udevadm control --reload-rules && sudo udevadm trigger
   ```

   *You will need to log out and back in for group changes to take effect if doing this manually.*

</details>

---

## ⌨️ How to Use

| Hotkey | Action |
| :--- | :--- |
| **`Super + V`** | Open Clipboard History |
| **`Ctrl + Alt + V`** | Alternative Open Shortcut |
| **`Esc`** | Close Window |
| **`Arrows / Tab`** | Navigate Items |
| **`Enter`** | Paste Selected Item |

### Advanced Usage
- **Paste GIFs:** Select a GIF, and it will be copied as a file URI. The app simulates `Ctrl+V` to paste it into apps like Discord or Telegram.
- **Pinning:** Click the pin icon on any item to prevent it from being auto-deleted when the history limit (50 items) is reached.

---

## 🛠️ Development

### Prerequisites

You need **Rust**, **Node.js (v20+)**, and build tools.

```bash
# Clone repo
git clone https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux.git
cd win11-clipboard-history

# Install system dependencies (auto-detects distro)
make deps

# Install language tools
make rust
make node
source ~/.cargo/env
```

### Running in Dev Mode

Use the provided script to ensure the environment is clean (fixes issues with VS Code Snap version):

```bash
make dev
# or
./scripts/run-dev.sh
```

### Building for Production

```bash
make build
# Artifacts will be in src-tauri/target/release/bundle/
```

---

## 🔧 Troubleshooting

### App won't open with Super+V
1. Ensure the app is running: `pgrep -f win11-clipboard-history`
2. Check permissions: `ls -l /dev/input/event0`. The group should be `input` and your user should be in that group (`groups`).
3. Try the alternative hotkey: `Ctrl+Alt+V`.

### Pasting doesn't work
1. **Wayland:** Ensure `wl-clipboard` is installed: `sudo apt install wl-clipboard`.
2. **X11:** Ensure `xclip` is installed: `sudo apt install xclip`.
3. The app simulates `Ctrl+V`. Ensure the target app accepts this shortcut.

### Window appears on the wrong monitor
The app uses smart cursor tracking. If it appears incorrectly, try moving your mouse to the center of the desired screen and pressing the hotkey again.

---

## 🗑️ Uninstalling

**Ubuntu / Debian:**
```bash
sudo apt remove win11-clipboard-history
```

**Fedora:**
```bash
sudo dnf remove win11-clipboard-history
```

**Installer Script / AppImage:**
```bash
rm -f ~/.local/bin/win11-clipboard-history
rm -rf ~/.local/lib/win11-clipboard-history
rm -f ~/.config/autostart/win11-clipboard-history.desktop
```

---

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
