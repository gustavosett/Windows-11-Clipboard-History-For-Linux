## 📝 Description

This PR introduces a "Quick Select" feature that allows users to instantly paste items from the clipboard history using `Alt + 1` through `Alt + 9`. 

Key behaviors:
- **Alt Shortcuts**: Pressing `Alt + 1-9` while the clipboard window is open instantly pastes the corresponding item. This works even when the search bar is focused.
- **Search Compatible**: If the user has started typing a search query without modifiers, numeric keys function as normal text input.
- **Improved Navigation**: Arrow keys, Home, and End now work even when the search bar is focused.
- **Fast Paste**: Pressing `Enter` while inside the search bar instantly pastes the currently selected item.

## 🔗 Related Issue

- https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux/issues/188

## 🧪 Type of Change

- [x] ✨ New feature (non-breaking change that adds functionality)

## ✅ Checklist

- [x] My code follows the project's code style
- [x] I have run `make lint` and `make format`
- [x] I have tested my changes locally
- [ ] I have updated documentation as needed
- [x] My changes don't introduce new warnings
- [ ] I have tested on both X11 and Wayland (if applicable)

## 🖥️ Testing Environment

- **OS**: Linux
- **Desktop Environment**: GNOME
- **Display Server**: X11
- **GPU**: NVIDIA (proprietary drivers)

## 📋 Additional Notes

Silent implementation to avoid UI clutter.