# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.x.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability, please follow these steps:

### 🔒 Private Disclosure

**Do NOT open a public issue for security vulnerabilities.**

Instead, please report security issues via one of these methods:

1. **GitHub Security Advisory**: Use [GitHub's private vulnerability reporting](https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux/security/advisories/new)

2. **Email**: gustaavoribeeiro@hotmail.com

### 📝 What to Include

When reporting a vulnerability, please include:

- **Description** of the vulnerability
- **Steps to reproduce** the issue
- **Potential impact** of the vulnerability
- **Suggested fix** (if you have one)
- **Your contact information** for follow-up questions

### ⏱️ Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 1 week
- **Fix Timeline**: Depends on severity
  - Critical: 24-72 hours
  - High: 1 week
  - Medium: 2 weeks
  - Low: Next release

### 🎁 Recognition

We appreciate security researchers who help keep our project safe. With your permission, we will:

- Acknowledge your contribution in release notes
- Add you to our security hall of fame (if created in the future)

## Security Best Practices

When using Windows 11 Clipboard History For Linux:

1. **Keep Updated**: Always use the latest version
2. **Build from Source**: When possible, verify the source code
3. **Check Signatures**: Verify release artifacts when available
4. **Report Issues**: Help us by reporting any suspicious behavior

## Known Security Considerations

### Clipboard Data

- Clipboard history is stored **locally only**
- No data is transmitted over the network
- History is stored in memory (not persisted to disk by default)
- Sensitive data copied to clipboard will be stored in history

### Permissions

- **Global hotkey capture**: Required for global shortcuts (Super+V, Ctrl+Alt+V)
- **System tray**: For background operation
- **Clipboard access**: Core functionality

### Wayland Security

On Wayland, clipboard access follows the compositor's security model, which may restrict access to clipboard contents from background applications in some configurations.

---

Thank you for helping keep Windows 11 Clipboard History For Linux secure! 🔐
