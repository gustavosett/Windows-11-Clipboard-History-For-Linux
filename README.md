# 📋 Windows 11 Clipboard History For Linux

<div align="center">

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![Tauri](https://img.shields.io/badge/tauri-v2-blue.svg)
![Platform](https://img.shields.io/badge/platform-linux-lightgrey.svg)

**A beautiful, Windows 11-style Clipboard History Manager for Linux**

![Screenshot](docs/img/win11-clipboard-history.jpg)

Built with 🦀 Rust + ⚡ Tauri v2 + ⚛️ React + 🎨 Tailwind CSS

[Features](#-features) • [Installation](#-installation) • [Development](#-development) • [Contributing](#-contributing)

</div>

---

## ✨ Features

- 🎨 **Windows 11 Design** - Pixel-perfect recreation of the Win+V clipboard UI with Acrylic/Mica glassmorphism effects
- 🌙 **Dark/Light Mode** - Automatically detects system theme preference
- ⌨️ **Global Hotkey** - Press `Super+V` or `Ctrl+Alt+V` to open from anywhere
- 📌 **Pin Items** - Keep important clipboard entries at the top
- 🖼️ **Image Support** - Copy and paste images with preview thumbnails
- 🚀 **Blazing Fast** - Written in Rust for maximum performance
- 🔒 **Privacy First** - All data stays local on your machine
- 🖱️ **Smart Positioning** - Window appears at your cursor position
- 💨 **System Tray** - Runs silently in the background
- 🐧 **Wayland & X11** - Works on both display servers

## 🐧 Installation (For Users)

### The easiest way to get started (Recommended)

> **One-line install:**
> ```bash
> curl -sL https://clipboard.gustavosett.dev | bash
> ```
> *Note: This script automatically detects your distribution (Ubuntu, Debian, Fedora, etc.), installs the appropriate package, and configures the necessary permissions.*

Or

### Using package managers

> Download the latest release from GitHub and follow the installation instructions for your distribution in [RELEASES](https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux/releases)

### ⚠️ Important: Permissions Required

This application requires access to input devices for:
- **Global hotkeys** (Super+V / Ctrl+Alt+V) - uses `rdev` to capture keyboard events
- **Paste simulation** (Ctrl+V injection) - uses `enigo` with uinput
- **Window positioning** - cursor position detection

After installation:

1. **Log out and log back in** for the permissions to take effect
2. The installer automatically:
   - Adds your user to the `input` group
   - Creates udev rules for input device access
   - Loads the `uinput` kernel module

If you installed manually, run:
```bash
# Add user to input group
sudo usermod -aG input $USER

# Create udev rules
sudo tee /etc/udev/rules.d/99-win11-clipboard-input.rules << 'EOF'
KERNEL=="event*", SUBSYSTEM=="input", MODE="0660", GROUP="input"
KERNEL=="uinput", SUBSYSTEM=="misc", MODE="0660", GROUP="input", OPTIONS+="static_node=uinput"
EOF

# Load uinput module
sudo modprobe uinput
echo "uinput" | sudo tee /etc/modules-load.d/uinput.conf

# Reload udev rules
sudo udevadm control --reload-rules && sudo udevadm trigger

# Then log out and log back in
```


## 📦 Installation (For devs)

```bash
# Clone the repository
git clone https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux.git
cd win11-clipboard-history

# Install system dependencies (auto-detects your distro)
make deps

# Install Rust and Node.js if needed
make rust
make node
source ~/.cargo/env  # Reload shell environment

# Build and install
make build
sudo make install
```

### Distribution-Specific Dependencies

<details>
<summary><b>🟠 Ubuntu / Debian / Linux Mint / Pop!_OS</b></summary>

```bash
sudo apt update
sudo apt install -y \
    libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libxdo-dev \
    libgtk-3-dev \
    libglib2.0-dev \
    pkg-config
```

</details>

<details>
<summary><b>🔵 Fedora</b></summary>

```bash
sudo dnf install -y \
    webkit2gtk4.1-devel \
    openssl-devel \
    curl \
    wget \
    file \
    libappindicator-gtk3-devel \
    librsvg2-devel \
    libxdo-devel \
    gtk3-devel \
    glib2-devel \
    pkg-config \
    @development-tools
```

</details>

<details>
<summary><b>🟣 Arch Linux / Manjaro / EndeavourOS</b></summary>

```bash
sudo pacman -Syu --needed \
    webkit2gtk-4.1 \
    base-devel \
    curl \
    wget \
    file \
    openssl \
    libappindicator-gtk3 \
    librsvg \
    xdotool \
    gtk3 \
    glib2 \
    pkgconf
```

</details>

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Install Node.js (v18+)

```bash
# Using nvm (recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
source ~/.nvm/nvm.sh
nvm install 20
nvm use 20
```

### Build from Source

```bash
# Clone the repository
git clone https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux.git
cd win11-clipboard-history

# Install npm dependencies
npm install

# Build the application
npm run tauri:build

# The built packages will be in:
# - Binary: src-tauri/target/release/win11-clipboard-history
# - DEB package: src-tauri/target/release/bundle/deb/
# - RPM package: src-tauri/target/release/bundle/rpm/
# - AppImage: src-tauri/target/release/bundle/appimage/
```

## 🛠️ Development

### Quick Start

```bash
# Install dependencies
npm install

# Run in development mode (hot reload enabled)
make dev
# OR
./scripts/run-dev.sh
```

> **Note for VS Code Snap users**: If you're using VS Code installed via Snap, use `make dev` or `./scripts/run-dev.sh` instead of `npm run tauri:dev` directly. This script cleans the environment to avoid library conflicts.

### Makefile Commands

| Command | Description |
|---------|-------------|
| `make help` | Show all available commands |
| `make deps` | Install system dependencies (auto-detect distro) |
| `make deps-ubuntu` | Install dependencies for Ubuntu/Debian |
| `make deps-fedora` | Install dependencies for Fedora |
| `make deps-arch` | Install dependencies for Arch Linux |
| `make rust` | Install Rust via rustup |
| `make node` | Install Node.js via nvm |
| `make check-deps` | Verify all dependencies are installed |
| `make dev` | Run in development mode |
| `make build` | Build production release |
| `make install` | Install to system (requires sudo) |
| `make uninstall` | Remove from system (requires sudo) |
| `make clean` | Remove build artifacts |
| `make lint` | Run linters |
| `make format` | Format code |

### npm Scripts

| Command | Description |
|---------|-------------|
| `npm run dev` | Start Vite dev server (frontend only) |
| `npm run tauri:dev` | Start full Tauri development mode |
| `npm run tauri:build` | Build production release |
| `npm run build` | Build frontend only |
| `npm run lint` | Run ESLint |
| `npm run format` | Format code with Prettier |

### Project Structure

```
win11-clipboard-history/
├── src/                      # React frontend
│   ├── components/           # UI components
│   │   ├── EmptyState.tsx    # Empty history state
│   │   ├── Header.tsx        # App header with actions
│   │   ├── HistoryItem.tsx   # Clipboard item card
│   │   └── TabBar.tsx        # Tab navigation
│   ├── hooks/                # React hooks
│   │   ├── useClipboardHistory.ts
│   │   └── useDarkMode.ts
│   ├── types/                # TypeScript types
│   │   └── clipboard.ts
│   ├── App.tsx               # Main app component
│   ├── index.css             # Global styles + Tailwind
│   └── main.tsx              # Entry point
├── src-tauri/                # Rust backend
│   ├── src/
│   │   ├── main.rs           # App setup, tray, commands
│   │   ├── lib.rs            # Library exports
│   │   ├── clipboard_manager.rs  # Clipboard operations
│   │   └── hotkey_manager.rs     # Global shortcuts
│   ├── capabilities/         # Tauri permissions
│   ├── icons/                # App icons
│   ├── Cargo.toml            # Rust dependencies
│   └── tauri.conf.json       # Tauri configuration
├── scripts/
│   └── run-dev.sh            # Clean environment dev script
├── Makefile                  # Build automation
├── tailwind.config.js        # Win11 theme config
├── vite.config.ts            # Vite configuration
└── package.json              # Node dependencies
```

### Global Hotkey Permissions

On Linux with X11, global keyboard capture may require the user to be in the `input` group:

```bash
sudo usermod -aG input $USER
# Log out and back in for changes to take effect
```

On Wayland, permissions are typically handled automatically by the compositor.

## 🐧 Platform Support

### Display Servers

| Display Server | Status | Notes |
|----------------|--------|-------|
| X11 | ✅ Full support | Global hotkeys work via rdev |
| Wayland | ✅ Full support | Uses wl-clipboard for clipboard access |

### Tested Distributions

| Distribution | Version | Status |
|--------------|---------|--------|
| Ubuntu | 22.04+ | ✅ Tested |
| Debian | 12+ | ✅ Tested |
| Fedora | 38+ | ✅ Tested |
| Arch Linux | Rolling | ✅ Tested |
| Manjaro | Latest | ✅ Tested |
| Linux Mint | 21+ | ✅ Tested |
| Pop!_OS | 22.04+ | ✅ Tested |

## 🎨 Customization

### Changing the Hotkey

Edit `src-tauri/src/hotkey_manager.rs` to modify the global shortcut:

```rust
// Current: Super+V or Ctrl+Alt+V
Key::KeyV => {
    if super_pressed || (ctrl_pressed && alt_pressed) {
        callback();
    }
}
```

### Theme Colors

The Windows 11 color palette is defined in `tailwind.config.js`:

```js
colors: {
  win11: {
    'bg-primary': '#202020',
    'bg-accent': '#0078d4',
    // ... customize as needed
  }
}
```

## 🔧 Troubleshooting

### Application won't start

1. **Check dependencies**: Run `make check-deps` to verify all dependencies are installed
2. **Wayland clipboard issues**: Ensure `wl-clipboard` is installed for Wayland support
3. **VS Code Snap conflict**: Use `make dev` or `./scripts/run-dev.sh` instead of `npm run tauri:dev`

### Global hotkey not working

1. **X11**: Add user to input group: `sudo usermod -aG input $USER`
2. **Wayland**: Some compositors may require additional permissions
3. Try alternative hotkey `Ctrl+Alt+V` instead of `Super+V`

### Window not showing at cursor position

This may occur on some Wayland compositors. The window will fallback to a default position.

## 🤝 Contributing

Contributions are welcome! Here's how you can help:

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Commit** your changes (`git commit -m 'feat: add amazing feature'`)
4. **Push** to the branch (`git push origin feature/amazing-feature`)
5. **Open** a Pull Request

### Development Guidelines

- Follow the existing code style
- Run `make lint` and `make format` before committing
- Write meaningful commit messages
- Add tests for new features
- Update documentation as needed

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Tauri](https://tauri.app/) - For the amazing Rust-based framework
- [Windows 11](https://www.microsoft.com/windows/windows-11) - For the beautiful design inspiration
- [rdev](https://github.com/Narsil/rdev) - For global keyboard capture
- [arboard](https://github.com/1Password/arboard) - For cross-platform clipboard access
- [wl-clipboard-rs](https://github.com/YaLTeR/wl-clipboard-rs) - For Wayland clipboard support

---

<div align="center">

**If you find this project useful, please consider giving it a ⭐!**

Made with ❤️ for the Linux community

</div>
