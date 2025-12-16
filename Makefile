# Windows 11 Clipboard History For Linux - Makefile
# Cross-distro build and install for Ubuntu, Debian, Fedora, and Arch Linux

SHELL := /bin/bash
APP_NAME := win11-clipboard-history
VERSION := 1.0.0
PREFIX ?= /usr/local
BINDIR := $(PREFIX)/bin
DATADIR := $(PREFIX)/share
DESTDIR ?=

# Detect distro
DISTRO := $(shell \
	if [ -f /etc/os-release ]; then \
		. /etc/os-release && echo $$ID; \
	elif [ -f /etc/debian_version ]; then \
		echo "debian"; \
	elif [ -f /etc/fedora-release ]; then \
		echo "fedora"; \
	elif [ -f /etc/arch-release ]; then \
		echo "arch"; \
	else \
		echo "unknown"; \
	fi)

# Colors for pretty output
CYAN := \033[0;36m
GREEN := \033[0;32m
YELLOW := \033[0;33m
RED := \033[0;31m
RESET := \033[0m

.PHONY: all help deps deps-ubuntu deps-debian deps-fedora deps-arch \
        rust node check-deps dev build install uninstall clean run \
        lint format test

all: build

help:
	@echo -e "$(CYAN)╔════════════════════════════════════════════════════════════════╗$(RESET)"
	@echo -e "$(CYAN)║     Windows 11 Clipboard History For Linux - Build Commands                   ║$(RESET)"
	@echo -e "$(CYAN)╚════════════════════════════════════════════════════════════════╝$(RESET)"
	@echo ""
	@echo -e "$(GREEN)Setup:$(RESET)"
	@echo "  make deps        - Install system dependencies (auto-detect distro)"
	@echo "  make deps-ubuntu - Install dependencies for Ubuntu"
	@echo "  make deps-debian - Install dependencies for Debian"
	@echo "  make deps-fedora - Install dependencies for Fedora"
	@echo "  make deps-arch   - Install dependencies for Arch Linux"
	@echo "  make rust        - Install Rust via rustup"
	@echo "  make node        - Install Node.js via nvm"
	@echo ""
	@echo -e "$(GREEN)Development:$(RESET)"
	@echo "  make dev         - Run in development mode (hot reload)"
	@echo "  make run         - Run the development version (clean env)"
	@echo "  make build       - Build production release"
	@echo "  make lint        - Run linters"
	@echo "  make format      - Format code"
	@echo ""
	@echo -e "$(GREEN)Installation:$(RESET)"
	@echo "  make install     - Install to system (requires sudo)"
	@echo "  make uninstall   - Remove from system (requires sudo)"
	@echo ""
	@echo -e "$(GREEN)Maintenance:$(RESET)"
	@echo "  make clean       - Remove build artifacts"
	@echo "  make check-deps  - Verify all dependencies are installed"
	@echo ""
	@echo -e "$(YELLOW)Detected distro: $(DISTRO)$(RESET)"

# ============================================================================
# Dependencies
# ============================================================================

deps:
	@echo -e "$(CYAN)Detected distribution: $(DISTRO)$(RESET)"
ifeq ($(DISTRO),ubuntu)
	@$(MAKE) deps-ubuntu
else ifeq ($(DISTRO),debian)
	@$(MAKE) deps-debian
else ifeq ($(DISTRO),fedora)
	@$(MAKE) deps-fedora
else ifeq ($(DISTRO),arch)
	@$(MAKE) deps-arch
else ifeq ($(DISTRO),manjaro)
	@$(MAKE) deps-arch
else ifeq ($(DISTRO),endeavouros)
	@$(MAKE) deps-arch
else ifeq ($(DISTRO),linuxmint)
	@$(MAKE) deps-ubuntu
else ifeq ($(DISTRO),pop)
	@$(MAKE) deps-ubuntu
else
	@echo -e "$(RED)Unknown distribution: $(DISTRO)$(RESET)"
	@echo "Please install dependencies manually. See README.md"
	@exit 1
endif

deps-ubuntu deps-debian:
	@echo -e "$(CYAN)Installing dependencies for Ubuntu/Debian...$(RESET)"
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
		xclip \
		wl-clipboard \
		pkg-config
	@echo -e "$(GREEN)✓ Dependencies installed successfully$(RESET)"

deps-fedora:
	@echo -e "$(CYAN)Installing dependencies for Fedora...$(RESET)"
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
		xclip \
		wl-clipboard \
		pkg-config \
		@development-tools
	@echo -e "$(GREEN)✓ Dependencies installed successfully$(RESET)"

deps-arch:
	@echo -e "$(CYAN)Installing dependencies for Arch Linux...$(RESET)"
	sudo pacman -Syu --needed --noconfirm \
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
		xclip \
		wl-clipboard \
		pkgconf
	@echo -e "$(GREEN)✓ Dependencies installed successfully$(RESET)"

rust:
	@echo -e "$(CYAN)Installing Rust via rustup...$(RESET)"
	@if command -v rustc &> /dev/null; then \
		echo -e "$(YELLOW)Rust is already installed: $$(rustc --version)$(RESET)"; \
	else \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; \
		echo -e "$(GREEN)✓ Rust installed. Run 'source ~/.cargo/env' to update PATH$(RESET)"; \
	fi

node:
	@echo -e "$(CYAN)Installing Node.js via nvm...$(RESET)"
	@if command -v node &> /dev/null; then \
		echo -e "$(YELLOW)Node.js is already installed: $$(node --version)$(RESET)"; \
	else \
		curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash; \
		export NVM_DIR="$$HOME/.nvm"; \
		[ -s "$$NVM_DIR/nvm.sh" ] && \. "$$NVM_DIR/nvm.sh"; \
		nvm install 20; \
		echo -e "$(GREEN)✓ Node.js installed$(RESET)"; \
	fi

check-deps:
	@echo -e "$(CYAN)Checking dependencies...$(RESET)"
	@echo ""
	@echo "System tools:"
	@command -v rustc &> /dev/null && echo -e "  $(GREEN)✓$(RESET) Rust: $$(rustc --version)" || echo -e "  $(RED)✗$(RESET) Rust: not found"
	@command -v cargo &> /dev/null && echo -e "  $(GREEN)✓$(RESET) Cargo: $$(cargo --version)" || echo -e "  $(RED)✗$(RESET) Cargo: not found"
	@command -v node &> /dev/null && echo -e "  $(GREEN)✓$(RESET) Node.js: $$(node --version)" || echo -e "  $(RED)✗$(RESET) Node.js: not found"
	@command -v npm &> /dev/null && echo -e "  $(GREEN)✓$(RESET) npm: $$(npm --version)" || echo -e "  $(RED)✗$(RESET) npm: not found"
	@echo ""
	@echo "Libraries:"
	@pkg-config --exists webkit2gtk-4.1 2>/dev/null && echo -e "  $(GREEN)✓$(RESET) webkit2gtk-4.1" || echo -e "  $(RED)✗$(RESET) webkit2gtk-4.1"
	@pkg-config --exists gtk+-3.0 2>/dev/null && echo -e "  $(GREEN)✓$(RESET) gtk+-3.0" || echo -e "  $(RED)✗$(RESET) gtk+-3.0"
	@pkg-config --exists glib-2.0 2>/dev/null && echo -e "  $(GREEN)✓$(RESET) glib-2.0" || echo -e "  $(RED)✗$(RESET) glib-2.0"
	@pkg-config --exists openssl 2>/dev/null && echo -e "  $(GREEN)✓$(RESET) openssl" || echo -e "  $(RED)✗$(RESET) openssl"
	@echo ""

# ============================================================================
# Development
# ============================================================================

dev: node_modules
	@echo -e "$(CYAN)Starting development server...$(RESET)"
	@./scripts/run-dev.sh

run: node_modules
	@echo -e "$(CYAN)Running with clean environment...$(RESET)"
	@./scripts/run-dev.sh

node_modules: package.json
	@echo -e "$(CYAN)Installing npm dependencies...$(RESET)"
	npm install
	@touch node_modules

# ============================================================================
# Build
# ============================================================================

build: node_modules
	@echo -e "$(CYAN)Building production release...$(RESET)"
	npm run tauri:build
	@echo -e "$(GREEN)✓ Build complete!$(RESET)"
	@echo -e "$(YELLOW)Packages available in: src-tauri/target/release/bundle/$(RESET)"

# ============================================================================
# Install / Uninstall
# ============================================================================

install:
	@echo -e "$(CYAN)Installing $(APP_NAME)...$(RESET)"
	@# Install binary to lib directory and wrapper to bin
	@mkdir -p $(DESTDIR)/usr/lib/$(APP_NAME)
	install -Dm755 src-tauri/target/release/$(APP_NAME) $(DESTDIR)/usr/lib/$(APP_NAME)/$(APP_NAME)-bin
	install -Dm755 src-tauri/bundle/linux/wrapper.sh $(DESTDIR)$(BINDIR)/$(APP_NAME)
	install -Dm644 src-tauri/icons/128x128.png $(DESTDIR)$(DATADIR)/icons/hicolor/128x128/apps/$(APP_NAME).png
	install -Dm644 src-tauri/icons/icon.png $(DESTDIR)$(DATADIR)/icons/hicolor/256x256/apps/$(APP_NAME).png
	@# Create comprehensive udev rules for input devices and uinput
	@mkdir -p /etc/udev/rules.d
	@echo '# udev rules for Windows 11 Clipboard History' > /etc/udev/rules.d/99-win11-clipboard-input.rules
	@echo '# Input devices - needed for evdev global hotkey detection' >> /etc/udev/rules.d/99-win11-clipboard-input.rules
	@echo 'KERNEL=="event*", SUBSYSTEM=="input", MODE="0660", GROUP="input"' >> /etc/udev/rules.d/99-win11-clipboard-input.rules
	@echo '# uinput device - needed for kernel-level keyboard simulation' >> /etc/udev/rules.d/99-win11-clipboard-input.rules
	@echo 'KERNEL=="uinput", SUBSYSTEM=="misc", MODE="0660", GROUP="input", OPTIONS+="static_node=uinput"' >> /etc/udev/rules.d/99-win11-clipboard-input.rules
	@# Load uinput module
	@modprobe uinput 2>/dev/null || true
	@# Ensure uinput loads on boot
	@mkdir -p /etc/modules-load.d
	@echo "uinput" > /etc/modules-load.d/uinput.conf
	@udevadm control --reload-rules 2>/dev/null || true
	@udevadm trigger 2>/dev/null || true
	@udevadm trigger --subsystem-match=misc --action=change 2>/dev/null || true
	@# Ensure input group exists and check/add user
	@getent group input >/dev/null 2>&1 || groupadd input
	@USER_IN_GROUP=false; \
	if [ -n "$$SUDO_USER" ]; then \
		if groups $$SUDO_USER 2>/dev/null | grep -q '\binput\b'; then \
			USER_IN_GROUP=true; \
			echo -e "$(GREEN)✓ User $$SUDO_USER already in input group$(RESET)"; \
		else \
			usermod -aG input $$SUDO_USER; \
			echo -e "$(GREEN)✓ Added $$SUDO_USER to input group$(RESET)"; \
		fi; \
	fi
	@# Create desktop entry
	@mkdir -p $(DESTDIR)$(DATADIR)/applications
	@echo "[Desktop Entry]" > $(DESTDIR)$(DATADIR)/applications/$(APP_NAME).desktop
	@echo "Name=Clipboard History" >> $(DESTDIR)$(DATADIR)/applications/$(APP_NAME).desktop
	@echo "Comment=Windows 11-style Clipboard History Manager" >> $(DESTDIR)$(DATADIR)/applications/$(APP_NAME).desktop
	@echo "Exec=$(BINDIR)/$(APP_NAME)" >> $(DESTDIR)$(DATADIR)/applications/$(APP_NAME).desktop
	@echo "Icon=$(APP_NAME)" >> $(DESTDIR)$(DATADIR)/applications/$(APP_NAME).desktop
	@echo "Terminal=false" >> $(DESTDIR)$(DATADIR)/applications/$(APP_NAME).desktop
	@echo "Type=Application" >> $(DESTDIR)$(DATADIR)/applications/$(APP_NAME).desktop
	@echo "Categories=Utility;" >> $(DESTDIR)$(DATADIR)/applications/$(APP_NAME).desktop
	@echo "Keywords=clipboard;history;paste;copy;" >> $(DESTDIR)$(DATADIR)/applications/$(APP_NAME).desktop
	@echo "StartupWMClass=$(APP_NAME)" >> $(DESTDIR)$(DATADIR)/applications/$(APP_NAME).desktop
	@# Ask user about autostart and create entry if desired
	@if [ -n "$$SUDO_USER" ]; then \
		AUTOSTART_DIR=$$(getent passwd $$SUDO_USER | cut -d: -f6)/.config/autostart; \
		AUTOSTART_FILE="$$AUTOSTART_DIR/$(APP_NAME).desktop"; \
		if [ -f "$$AUTOSTART_FILE" ]; then \
			echo -e "$(GREEN)✓ Autostart already configured$(RESET)"; \
		else \
			echo ""; \
			echo -e "$(CYAN)Would you like the app to start automatically on login? [Y/n]$(RESET)"; \
			read -r AUTOSTART_RESPONSE; \
			case "$$AUTOSTART_RESPONSE" in \
				[nN]|[nN][oO]) \
					echo -e "$(YELLOW)! Autostart skipped$(RESET)"; \
					;; \
				*) \
					mkdir -p "$$AUTOSTART_DIR"; \
					cp $(DESTDIR)$(DATADIR)/applications/$(APP_NAME).desktop "$$AUTOSTART_FILE"; \
					echo "X-GNOME-Autostart-enabled=true" >> "$$AUTOSTART_FILE"; \
					echo "X-GNOME-Autostart-Delay=2" >> "$$AUTOSTART_FILE"; \
					chown -R $$SUDO_USER:$$SUDO_USER "$$AUTOSTART_DIR"; \
					echo -e "$(GREEN)✓ Autostart enabled$(RESET)"; \
					;; \
			esac; \
		fi; \
	fi
	@# Grant immediate access with ACLs and determine if logout is needed
	@NEEDS_LOGOUT=false; \
	APP_LAUNCHED=false; \
	if [ -n "$$SUDO_USER" ]; then \
		if groups $$SUDO_USER 2>/dev/null | grep -q '\binput\b'; then \
			NEEDS_LOGOUT=false; \
		elif command -v setfacl >/dev/null 2>&1; then \
			for dev in /dev/input/event*; do \
				setfacl -m "u:$$SUDO_USER:rw" "$$dev" 2>/dev/null || true; \
			done; \
			setfacl -m "u:$$SUDO_USER:rw" /dev/uinput 2>/dev/null || true; \
			echo -e "$(GREEN)✓ Granted immediate access$(RESET)"; \
			NEEDS_LOGOUT=false; \
		else \
			NEEDS_LOGOUT=true; \
		fi; \
	fi; \
	if [ "$$NEEDS_LOGOUT" = false ] && [ -n "$$SUDO_USER" ] && command -v gtk-launch >/dev/null 2>&1; then \
		USER_ID=$$(id -u $$SUDO_USER 2>/dev/null); \
		if [ -n "$$USER_ID" ] && [ -n "$$DISPLAY" -o -n "$$WAYLAND_DISPLAY" ]; then \
			sudo -u $$SUDO_USER \
				DISPLAY="$${DISPLAY:-:0}" \
				WAYLAND_DISPLAY="$$WAYLAND_DISPLAY" \
				XDG_RUNTIME_DIR="/run/user/$$USER_ID" \
				DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$$USER_ID/bus" \
				gtk-launch $(APP_NAME) 2>/dev/null & \
			sleep 1; \
			if pgrep -u $$SUDO_USER -x "$(APP_NAME)" > /dev/null 2>&1; then \
				APP_LAUNCHED=true; \
			fi; \
		fi; \
	fi; \
	echo -e "$(GREEN)✓ Installed successfully$(RESET)"; \
	echo ""; \
	if [ "$$NEEDS_LOGOUT" = true ]; then \
		echo -e "$(YELLOW)╔════════════════════════════════════════════════════════════════╗$(RESET)"; \
		echo -e "$(YELLOW)║     ⚠ Please log out and log back in for permissions          ║$(RESET)"; \
		echo -e "$(YELLOW)╚════════════════════════════════════════════════════════════════╝$(RESET)"; \
		echo ""; \
		echo "After logging back in, the app will start automatically."; \
	elif [ "$$APP_LAUNCHED" = true ]; then \
		echo -e "$(GREEN)╔════════════════════════════════════════════════════════════════╗$(RESET)"; \
		echo -e "$(GREEN)║     ✓ App is now running! Press Super+V to open.              ║$(RESET)"; \
		echo -e "$(GREEN)╚════════════════════════════════════════════════════════════════╝$(RESET)"; \
	else \
		echo -e "$(GREEN)╔════════════════════════════════════════════════════════════════╗$(RESET)"; \
		echo -e "$(GREEN)║     ✓ Installed! Find 'Clipboard History' in your app menu.   ║$(RESET)"; \
		echo -e "$(GREEN)╚════════════════════════════════════════════════════════════════╝$(RESET)"; \
	fi
	@echo ""

uninstall:
	@echo -e "$(CYAN)Uninstalling $(APP_NAME)...$(RESET)"
	@# Stop any running instances first
	@if [ -n "$$SUDO_USER" ]; then \
		pkill -u $$SUDO_USER -x "$(APP_NAME)" 2>/dev/null || true; \
	fi
	@pkill -x "$(APP_NAME)" 2>/dev/null || true
	rm -f $(DESTDIR)$(BINDIR)/$(APP_NAME)
	rm -rf $(DESTDIR)/usr/lib/$(APP_NAME)
	rm -f $(DESTDIR)$(DATADIR)/icons/hicolor/128x128/apps/$(APP_NAME).png
	rm -f $(DESTDIR)$(DATADIR)/icons/hicolor/256x256/apps/$(APP_NAME).png
	rm -f $(DESTDIR)$(DATADIR)/applications/$(APP_NAME).desktop
	rm -f /etc/udev/rules.d/99-win11-clipboard-input.rules
	rm -f /etc/modules-load.d/uinput.conf
	@# Remove autostart entry for the user
	@if [ -n "$$SUDO_USER" ]; then \
		AUTOSTART_FILE=$$(getent passwd $$SUDO_USER | cut -d: -f6)/.config/autostart/$(APP_NAME).desktop; \
		rm -f "$$AUTOSTART_FILE" 2>/dev/null || true; \
	fi
	@# Also remove for all users
	@for user_home in /home/*; do \
		if [ -d "$$user_home" ]; then \
			rm -f "$$user_home/.config/autostart/$(APP_NAME).desktop" 2>/dev/null || true; \
		fi; \
	done
	@echo -e "$(GREEN)✓ Uninstalled successfully$(RESET)"

# ============================================================================
# Code Quality
# ============================================================================

lint:
	@echo -e "$(CYAN)Running linters...$(RESET)"
	npm run lint
	cd src-tauri && cargo clippy -- -D warnings

format:
	@echo -e "$(CYAN)Formatting code...$(RESET)"
	npm run format
	cd src-tauri && cargo fmt

# ============================================================================
# Clean
# ============================================================================

clean:
	@echo -e "$(CYAN)Cleaning build artifacts...$(RESET)"
	rm -rf node_modules
	rm -rf dist
	rm -rf src-tauri/target
	@echo -e "$(GREEN)✓ Cleaned$(RESET)"
