<div align="center">

<img width="52" height="52" alt="logo" src="https://github.com/user-attachments/assets/4534e915-5d83-45f3-9f09-48a0f94b1d9a" />


# Windows 11 Clipboard History For Linux

[Website](https://clipboard.gustavosett.dev) • [Report Bug](https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux/issues) • [Request Feature](https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux/discussions/new?category=ideas)

**The aesthetic, feature-rich clipboard manager your Linux desktop deserves.**

[![License](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)
[![Total Downloads](https://img.shields.io/endpoint?url=https://clipboard.gustavosett.workers.dev/&style=for-the-badge&logo=cloudsmith&logoColor=white)](https://broadcasts.cloudsmith.com/gustavosett/clipboard-manager)
[![Tauri](https://img.shields.io/badge/Built_With-Tauri_v2-24C8D6?style=for-the-badge&logo=tauri&logoColor=white)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Powered_By-Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)

![App Screenshot](https://github.com/user-attachments/assets/74400c8b-9d7d-49ce-8de7-45dfd556e256)

</div>

---

## ⚡ Quick Start (Recommended)

Get up and running in seconds. This script detects your distro, installs the app, and configures permissions automatically.

```bash
# Just copy and paste this into your terminal
curl -fsSL https://raw.githubusercontent.com/gustavosett/Windows-11-Clipboard-History-For-Linux/master/scripts/install.sh | bash
```

> **Note:** No logout required! The installer uses ACLs to grant immediate access.

---

## 🌟 Why use this?

Most Linux clipboard managers are purely functional but lack visual appeal. This project brings the **modern, fluid design of Windows 11's clipboard history** to the Linux ecosystem, backed by the blazing speed of Rust.

| 😎 | 🔍 |
| --- | --- |
| **🐧 Universal Support** | Works flawlessly on both **Wayland** & **X11**. |
| **⚡ Instant Access** | Opens instantly with `Super+V` or `Ctrl+Alt+V`. |
| **🧠 Smart Positioning** | The window follows your mouse cursor across multiple monitors. |
| **📌 Pin & Sync** | Pin important snippets to keep them at the top. |
| **🎬 GIF Integration** | Search Tenor and paste GIFs directly into Discord, Slack, etc. |
| **🤩 Emoji Picker** | A built-in, searchable emoji keyboard. |
| **🛡️ Privacy First** | Your history is stored locally. No data leaves your machine. |

---

## ⌨️ Shortcuts & Usage

| Key | Action |
| --- | --- |
| <kbd>Super</kbd> + <kbd>V</kbd> | **Open Clipboard History** |
| <kbd>Ctrl</kbd> + <kbd>Alt</kbd> + <kbd>V</kbd> | Alternative Shortcut |
| <kbd>Enter</kbd> | Paste Selected Item |
| <kbd>Esc</kbd> | Close Window |

> **Pro Tip:** Need to paste a GIF? Just select it! The app simulates `Ctrl+V` to paste the file URI directly into your chat apps.

---

## 📦 Detailed Installation

Prefer to install manually? We support all major distributions.

<details>
<summary><b>Debian / Ubuntu / Mint / Pop!_OS</b></summary>

> **Recommended:** Use the APT repository for automatic updates.

```bash
# 1. Add Repository
curl -1sLf 'https://dl.cloudsmith.io/public/gustavosett/clipboard-manager/setup.deb.sh' | sudo -E bash

# 2. Install
sudo apt update && sudo apt install win11-clipboard-history

# 3. Grant Permissions (One-time)
sudo setfacl -m u:$USER:rw /dev/uinput

```

</details>

<details>
<summary><b>Fedora / RHEL / CentOS</b></summary>

```bash
# 1. Add Repository
curl -1sLf 'https://dl.cloudsmith.io/public/gustavosett/clipboard-manager/setup.rpm.sh' | sudo -E bash

# 2. Install
sudo dnf install win11-clipboard-history

# 3. Grant Permissions (One-time)
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

> ## Some features are disabled; we strongly recommend the complete installation.

1. Download the `.AppImage` from [Releases](https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux/releases).
2. Make it executable: `chmod +x win11-clipboard-history_*.AppImage`
3. Grant permissions: `sudo setfacl -m u:$USER:rw /dev/uinput`
4. Register the command that you want in your system to open the AppImage
```
KEYBOARD SETTINGS -> SHORTCUTS -> NEW SHORTCUT -> Super+V -> ./my_awesome_folder/win11-clipboard-history.AppImage
```

</details>

---

## 🔧 Troubleshooting

<details>
<summary><b>Shortcut (Super+V) isn't working</b></summary>

1. Ensure the app is running: `pgrep -f win11-clipboard-history-bin`
2. If running, try resetting the config:
```bash
rm ~/.config/win11-clipboard-history/setup.json
win11-clipboard-history

```


3. **Conflicts:** GNOME and other DEs often reserve `Super+V`. The app's **Setup Wizard** usually fixes this, but you can manually unbind `Super+V` in your system keyboard settings.

</details>

<details>
<summary><b>Transparency Issues (NVIDIA / AppImage)</b></summary>

If you see a black background or flickering, use the compatibility mode:

```bash
# Force NVIDIA workaround
IS_NVIDIA=1 win11-clipboard-history

# Force AppImage workaround
IS_APPIMAGE=1 win11-clipboard-history

```

</details>

---

## 🛠️ For Developers

Want to hack on the code?

**Tech Stack:** `Rust` + `Tauri v2` + `React` + `Tailwind CSS` + `Linux`

<div align="center">
  <a href="https://skillicons.dev">
    <img src="https://skillicons.dev/icons?i=rust,tauri,react,ts,tailwind,linux" />
  </a>
</div>

```bash
# 1. Clone
git clone https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux.git
cd Windows-11-Clipboard-History-For-Linux

# 2. Install Deps
make deps && make rust && make node
source ~/.cargo/env

# 3. Run Dev Mode
make dev

```

---

## ✨ Contributors

Thanks goes to these wonderful people ([emoji key](https://allcontributors.org/docs/en/emoji-key)):

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/freshCoder21313"><img src="https://avatars.githubusercontent.com/u/151538542?v=4?s=100" width="100px;" alt="freshCoder21313"/><br /><sub><b>freshCoder21313</b></sub></a><br /><a href="#data-freshCoder21313" title="Data">🔣</a> <a href="https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux/gustavosett/Windows-11-Clipboard-History-For-Linux/commits?author=freshCoder21313" title="Code">💻</a> <a href="#design-freshCoder21313" title="Design">🎨</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Tallin-Boston-Technology"><img src="https://avatars.githubusercontent.com/u/247321893?v=4?s=100" width="100px;" alt="Tallin-Boston-Technology"/><br /><sub><b>Tallin-Boston-Technology</b></sub></a><br /><a href="#ideas-Tallin-Boston-Technology" title="Ideas, Planning, & Feedback">🤔</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/rorar"><img src="https://avatars.githubusercontent.com/u/44790144?v=4?s=100" width="100px;" alt="rorar"/><br /><sub><b>rorar</b></sub></a><br /><a href="#ideas-rorar" title="Ideas, Planning, & Feedback">🤔</a> <a href="https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux/gustavosett/Windows-11-Clipboard-History-For-Linux/issues?q=author%3Arorar" title="Bug reports">🐛</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/sosadsonar"><img src="https://avatars.githubusercontent.com/u/120033042?v=4?s=100" width="100px;" alt="sonarx"/><br /><sub><b>sonarx</b></sub></a><br /><a href="#ideas-sosadsonar" title="Ideas, Planning, & Feedback">🤔</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://oleksandrdev.com/"><img src="https://avatars.githubusercontent.com/u/47930925?v=4?s=100" width="100px;" alt="Oleksandr Romaniuk"/><br /><sub><b>Oleksandr Romaniuk</b></sub></a><br /><a href="https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux/gustavosett/Windows-11-Clipboard-History-For-Linux/issues?q=author%3Aolksndrdevhub" title="Bug reports">🐛</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Predrag"><img src="https://avatars.githubusercontent.com/u/460694?v=4?s=100" width="100px;" alt="Predrag"/><br /><sub><b>Predrag</b></sub></a><br /><a href="https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux/gustavosett/Windows-11-Clipboard-History-For-Linux/commits?author=Predrag" title="Code">💻</a> <a href="https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux/gustavosett/Windows-11-Clipboard-History-For-Linux/issues?q=author%3APredrag" title="Bug reports">🐛</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/henmalib"><img src="https://avatars.githubusercontent.com/u/68553709?v=4?s=100" width="100px;" alt="Hen"/><br /><sub><b>Hen</b></sub></a><br /><a href="https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux/gustavosett/Windows-11-Clipboard-History-For-Linux/issues?q=author%3Ahenmalib" title="Bug reports">🐛</a> <a href="https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux/gustavosett/Windows-11-Clipboard-History-For-Linux/commits?author=henmalib" title="Code">💻</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/e6ad2020"><img src="https://avatars.githubusercontent.com/u/119390190?v=4?s=100" width="100px;" alt="Eyad"/><br /><sub><b>Eyad</b></sub></a><br /><a href="https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux/gustavosett/Windows-11-Clipboard-History-For-Linux/issues?q=author%3Ae6ad2020" title="Bug reports">🐛</a> <a href="https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux/gustavosett/Windows-11-Clipboard-History-For-Linux/commits?author=e6ad2020" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://alexandre-pommier.com"><img src="https://avatars.githubusercontent.com/u/69145792?v=4?s=100" width="100px;" alt="Kinou"/><br /><sub><b>Kinou</b></sub></a><br /><a href="https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux/gustavosett/Windows-11-Clipboard-History-For-Linux/issues?q=author%3Akinou-p" title="Bug reports">🐛</a> <a href="https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux/gustavosett/Windows-11-Clipboard-History-For-Linux/commits?author=kinou-p" title="Code">💻</a> <a href="#question-kinou-p" title="Answering Questions">💬</a> <a href="#design-kinou-p" title="Design">🎨</a></td>
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

<div align="center">
<br />

# Like this project?

<img alt="give it a star" src="https://github.com/user-attachments/assets/0e4e0804-095a-469c-aca5-e559202840f7" />

---

<img alt="Static Badge" src="https://img.shields.io/badge/OSS%20hosting%20by-cloudsmith-blue?logo=cloudsmith&style=flat-square&link=https%3A%2F%2Fcloudsmith.com">
</img>


Package repository hosting is graciously provided by [Cloudsmith](https://cloudsmith.com).
Cloudsmith is the only fully hosted, cloud-native, universal package management solution, that
enables your organization to create, store and share packages in any format, to any place, with total
confidence.
</div>
