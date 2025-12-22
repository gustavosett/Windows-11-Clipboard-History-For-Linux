# Windows 11 Clipboard History For Linux - Makefile
# Cross-distro build and install for Ubuntu, Debian, Fedora, and Arch Linux
#
# Note: PREFIX defaults to /usr/local for manual installs (Linux convention).
#       Package managers (deb/rpm) use PREFIX=/usr automatically during build.
#       To install system-wide like a package: sudo make install PREFIX=/usr

SHELL := /bin/bash
APP_NAME := win11-clipboard-history
VERSION := 1.0.0
PREFIX ?= /usr/local
LIBDIR := $(PREFIX)/lib
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
        rust node check-deps dev build install uninstall clean clean-first-run run \
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
	@echo "  make clean          - Remove build artifacts and first-run config"
	@echo "  make clean-first-run - Reset first-run config (shows Setup Wizard again)"
	@echo "  make check-deps     - Verify all dependencies are installed"
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
	@# Install binary to lib directory
	@mkdir -p $(DESTDIR)$(LIBDIR)/$(APP_NAME)
	install -Dm755 src-tauri/target/release/$(APP_NAME)-bin $(DESTDIR)$(LIBDIR)/$(APP_NAME)/$(APP_NAME)-bin
	@# Install wrapper script to bin
	install -Dm755 src-tauri/bundle/linux/wrapper.sh $(DESTDIR)$(BINDIR)/$(APP_NAME)
	@# Install icons
	install -Dm644 src-tauri/icons/128x128.png $(DESTDIR)$(DATADIR)/icons/hicolor/128x128/apps/$(APP_NAME).png
	install -Dm644 src-tauri/icons/icon.png $(DESTDIR)$(DATADIR)/icons/hicolor/256x256/apps/$(APP_NAME).png
	@# Create udev rules for input devices and uinput
	@mkdir -p $(DESTDIR)/etc/udev/rules.d
	install -Dm644 src-tauri/bundle/linux/99-win11-clipboard-input.rules $(DESTDIR)/etc/udev/rules.d/
	@# Ensure uinput loads on boot (use same filename as postinst/postrm)
	@mkdir -p $(DESTDIR)/etc/modules-load.d
	@echo "uinput" > $(DESTDIR)/etc/modules-load.d/win11-clipboard.conf
	@# Load module and reload udev only when installing on live system (not in DESTDIR/fakeroot)
	@if [ -z "$$(printf '%s' "$(DESTDIR)")" ]; then \
		modprobe uinput 2>/dev/null || true; \
		udevadm control --reload-rules 2>/dev/null || true; \
		udevadm trigger 2>/dev/null || true; \
		udevadm trigger --subsystem-match=misc --action=change 2>/dev/null || true; \
	fi
	@# Ensure input group exists and add user (only on live system, not in DESTDIR/fakeroot)
	@if [ -z "$$(printf '%s' "$(DESTDIR)")" ]; then \
		getent group input >/dev/null 2>&1 || groupadd input; \
		if [ -n "$$SUDO_USER" ]; then \
			if groups $$SUDO_USER 2>/dev/null | grep -q '\binput\b'; then \
				echo -e "$(GREEN)✓ User $$SUDO_USER already in input group$(RESET)"; \
			else \
				usermod -aG input $$SUDO_USER; \
				echo -e "$(GREEN)✓ Added $$SUDO_USER to input group$(RESET)"; \
			fi; \
		fi; \
	fi
	@# Install desktop entry
	@mkdir -p $(DESTDIR)$(DATADIR)/applications
	install -Dm644 src-tauri/bundle/linux/$(APP_NAME).desktop $(DESTDIR)$(DATADIR)/applications/
	@update-desktop-database $(DESTDIR)$(DATADIR)/applications 2>/dev/null || true
	@gtk-update-icon-cache -f -t $(DESTDIR)$(DATADIR)/icons/hicolor 2>/dev/null || true
	@echo -e "$(GREEN)✓ Installed successfully$(RESET)"
	@echo ""
	@echo -e "$(GREEN)╔════════════════════════════════════════════════════════════════╗$(RESET)"
	@echo -e "$(GREEN)║     ✓ Installed! Find 'Clipboard History' in your app menu.    ║$(RESET)"
	@echo -e "$(GREEN)╚════════════════════════════════════════════════════════════════╝$(RESET)"
	@echo ""
	@echo "The app will guide you through setup on first run."
	@echo "You may need to log out and back in for input permissions to take effect."
	@echo ""

uninstall:
	@echo -e "$(CYAN)Uninstalling $(APP_NAME)...$(RESET)"
	@# Stop any running instances first
	@if [ -n "$$SUDO_USER" ]; then \
		pkill -u $$SUDO_USER -f "$(APP_NAME)-bin" 2>/dev/null || true; \
	fi
	@pkill -f "$(APP_NAME)-bin" 2>/dev/null || true
	rm -f $(DESTDIR)$(BINDIR)/$(APP_NAME)
	rm -rf $(DESTDIR)$(LIBDIR)/$(APP_NAME)
	rm -f $(DESTDIR)$(DATADIR)/icons/hicolor/128x128/apps/$(APP_NAME).png
	rm -f $(DESTDIR)$(DATADIR)/icons/hicolor/256x256/apps/$(APP_NAME).png
	rm -f $(DESTDIR)$(DATADIR)/applications/$(APP_NAME).desktop
	rm -f $(DESTDIR)/etc/udev/rules.d/99-win11-clipboard-input.rules
	rm -f $(DESTDIR)/etc/modules-load.d/uinput.conf
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
	@update-desktop-database $(DESTDIR)$(DATADIR)/applications 2>/dev/null || true
	@gtk-update-icon-cache -f -t $(DESTDIR)$(DATADIR)/icons/hicolor 2>/dev/null || true
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

clean-first-run:
	@echo -e "$(CYAN)Cleaning first-run config...$(RESET)"
	@rm -f ~/.config/win11-clipboard-history/setup.json
	@echo -e "$(GREEN)✓ First-run config cleaned (Setup Wizard will show on next launch)$(RESET)"

clean: clean-first-run
	@echo -e "$(CYAN)Cleaning build artifacts...$(RESET)"
	rm -rf node_modules
	rm -rf dist
	rm -rf src-tauri/target
	@echo -e "$(GREEN)✓ Cleaned$(RESET)"
