# Contributing to Windows 11 Clipboard History For Linux

First off, thank you for considering contributing to Windows 11 Clipboard History For Linux! 🎉

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Pull Request Process](#pull-request-process)
- [Style Guidelines](#style-guidelines)
- [Reporting Bugs](#reporting-bugs)
- [Suggesting Features](#suggesting-features)

## Code of Conduct

This project and everyone participating in it is governed by our commitment to creating a welcoming and inclusive environment. Please be respectful and constructive in all interactions.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR-USERNAME/Windows-11-Clipboard-History-For-Linux.git
   cd win11-clipboard-history-for-linux
   ```
3. **Add the upstream remote**:
   ```bash
   git remote add upstream https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux.git
   ```

## Development Setup

### Prerequisites

Make sure you have the required dependencies installed:

- **Rust 1.77+** (for autostart plugin compatibility)
- **Node.js 20+**
- System build dependencies

```bash
# Install system dependencies (auto-detects your distro)
make deps

# Install Rust and Node.js if needed
make rust
make node
source ~/.cargo/env

# Verify everything is installed
make check-deps
```

### Running in Development Mode

```bash
# Install npm dependencies
npm install

# Start development server with hot reload
make dev
```

## Making Changes

1. **Create a new branch** from `master`:
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/your-bug-fix
   ```

2. **Make your changes** and test them locally

3. **Run linters and formatters**:
   ```bash
   make lint
   make format
   ```

4. **Commit your changes** with a descriptive message:
   ```bash
   git commit -m "feat: add amazing new feature"
   # or
   git commit -m "fix: resolve clipboard paste issue on Wayland"
   ```

### Commit Message Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `style:` - Code style changes (formatting, etc.)
- `refactor:` - Code refactoring
- `perf:` - Performance improvements
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks

## Pull Request Process

1. **Update your branch** with the latest upstream changes:
   ```bash
   git fetch upstream
   git rebase upstream/master
   ```

2. **Push your branch** to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

3. **Create a Pull Request** on GitHub

4. **Fill out the PR template** completely

5. **Wait for review** - maintainers will review your PR and may request changes

### PR Requirements

- [ ] Code follows the project's style guidelines
- [ ] All linters pass (`make lint`)
- [ ] Changes are tested locally
- [ ] Documentation is updated if needed
- [ ] PR description clearly explains the changes

## Style Guidelines

### TypeScript/React

- Use functional components with hooks
- Follow existing component patterns
- Use TypeScript types (avoid `any`)
- Use meaningful variable and function names

### Rust

- Follow Rust idioms and best practices
- Use `cargo fmt` for formatting
- Address all `clippy` warnings
- Document public functions and modules

### CSS/Tailwind

- Use Tailwind utility classes
- Follow the Windows 11 design system defined in `tailwind.config.js`
- Support both light and dark modes

## Reporting Bugs

Before reporting a bug:

1. **Search existing issues** to avoid duplicates
2. **Try the latest version** - the bug might be fixed
3. **Collect information**:
   - OS and version
   - Desktop environment
   - Display server (X11/Wayland)
   - Steps to reproduce
   - Error messages/logs

Then [create a bug report](https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux/issues/new?template=bug_report.md).

## Suggesting Features

We welcome feature suggestions! Before suggesting:

1. **Check if it aligns** with the project's goal (Windows 11-style clipboard manager)
2. **Search existing issues** to avoid duplicates
3. **Consider implementation** - how complex would it be?

Then [create a feature request](https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux/issues/new?template=feature_request.md).

## Questions?

Feel free to [open a discussion](https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux/discussions) for questions or ideas.

---

Thank you for contributing! 🙏
