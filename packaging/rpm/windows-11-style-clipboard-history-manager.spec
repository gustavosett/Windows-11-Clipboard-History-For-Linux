%global app_name windows-11-style-clipboard-history-manager
%global app_binary windows-11-style-clipboard-history-manager-bin
%global app_libdir %{_lib}/%{app_name}

Name:           windows-11-style-clipboard-history-manager
Version:        2.5.0
Release:        1%{?dist}
Summary:        Windows 11-style clipboard history manager for Linux
License:        MIT
URL:            https://github.com/Mahdi-Arts/Windows-11-Style-Clipboard-History-Manager
Source0:        %{url}/releases/download/%{version}/%{name}-%{version}.tar.gz
BuildRequires:  cargo
BuildRequires:  pkgconfig(webkit2gtk-4.1)
BuildRequires:  pkgconfig(webkit2gtk-4.0)
BuildRequires:  pkgconfig(gtk+-3.0)
BuildRequires:  pkgconfig(libayatana-appindicator-gtk3)
BuildRequires:  pkgconfig(libappindicator-gtk3)
BuildRequires:  pkgconfig(libxdo)
BuildRequires:  openssl-devel
BuildRequires:  gcc
BuildRequires:  gcc-c++
BuildRequires:  make
Requires:       xclip
Requires:       xdotool
Requires:       wl-clipboard
Requires:       acl
Requires:       polkit
Requires:       webkit2gtk4.1%{?_isa}
Requires:       gtk3%{?_isa}
Requires:       libayatana-appindicator-gtk3%{?_isa}
Recommends:     libsecret-tools

%description
A fast, beautiful clipboard history manager for Linux (Wayland & X11),
inspired by Windows 11 Win+V.

Features:
* Wayland and X11 clipboard monitoring
* Super+V / Ctrl+Alt+V global shortcuts
* Persian + English interface
* Encrypted local history (ChaCha20-Poly1305)
* Emoji, kaomoji, and symbol pickers
* Fully offline — bundled fonts

%prep
%autosetup -p1

%build
# Build is done in CI/Release process
# This spec file is for RPM packaging verification
%make_build

%install
install -Dm755 %{_builddir}/%{name}-%{version}/src-tauri/target/release/%{app_binary} \
    %{buildroot}%{_libdir}/%{app_name}/%{app_binary}

install -Dm755 %{_builddir}/%{name}-%{version}/src-tauri/bundle/linux/wrapper.sh \
    %{buildroot}%{_bindir}/%{app_name}

install -Dm644 %{_builddir}/%{name}-%{version}/src-tauri/bundle/linux/*.rules \
    %{buildroot}%{_sysconfdir}/udev/rules.d/99-windows-11-style-clipboard-history-input.rules

install -Dm644 %{_builddir}/%{name}-%{version}/src-tauri/bundle/linux/*.desktop \
    %{buildroot}%{_datadir}/applications/io.github.mahdi-arts.clipboard-history.desktop

install -Dm644 %{_builddir}/%{name}-%{version}/icons/128x128.png \
    %{buildroot}%{_datadir}/icons/hicolor/128x128/apps/io.github.mahdi-arts.clipboard-history.png

install -Dm644 %{_builddir}/%{name}-%{version}/icons/icon.png \
    %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/io.github.mahdi-arts.clipboard-history.png

install -Dm644 %{_builddir}/%{name}-%{version}/icons/icon.svg \
    %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/io.github.mahdi-arts.clipboard-history.svg

install -Dm644 packaging/apparmor/%{app_name} \
    %{buildroot}%{_datadir}/doc/%{name}/apparmor/%{app_name}

%pre
# Pre-installation scriptlet
# Stop running instances
if pgrep -x "windows-11-style-clipboard-history-manager" > /dev/null 2>&1; then
    echo "Stopping running instances..."
    pkill -x "windows-11-style-clipboard-history-manager" || true
    sleep 2
fi

%post
# Post-installation scriptlet
# Reload udev and setup permissions
echo "Setting up /dev/uinput permissions..."
udevadm control --reload-rules 2>/dev/null || true
udevadm trigger --subsystem-match=input 2>/dev/null || true

# Get the user who invoked sudo/pkexec
if [ -n "$SUDO_USER" ]; then
    TARGET_USER="$SUDO_USER"
elif [ -n "$PKEXEC_USER" ]; then
    TARGET_USER="$PKEXEC_USER"
else
    TARGET_USER=$(whoami)
fi

# Set ACL on uinput for the user
if [ -e /dev/uinput ] && [ "$TARGET_USER" != "root" ]; then
    setfacl -m u:"$TARGET_USER":rw /dev/uinput 2>/dev/null || \
        echo "Warning: Could not set ACL. Run: sudo setfacl -m u:$TARGET_USER:rw /dev/uinput"
fi

# Update desktop database
update-desktop-database %{_datadir}/applications 2>/dev/null || true

echo "Installation complete!"
echo "Please log out and back in for /dev/uinput permissions to take effect."

%preun
# Pre-uninstallation scriptlet
if [ $1 -eq 0 ]; then
    # Only on final removal, not upgrade
    echo "Stopping running instances..."
    pkill -x "windows-11-style-clipboard-history-manager" 2>/dev/null || true
fi

%postun
# Post-uninstallation scriptlet
if [ $1 -eq 0 ]; then
    # Only on final removal
    udevadm control --reload-rules 2>/dev/null || true
    update-desktop-database %{_datadir}/applications 2>/dev/null || true
    
    echo "Application removed."
    echo "User data preserved at:"
    echo "  ~/.local/share/windows-11-style-clipboard-history-manager"
    echo "  ~/.config/windows-11-style-clipboard-history-manager"
fi

%files
%defattr(-,root,root,-)
%{_libdir}/%{app_name}/%{app_binary}
%{_bindir}/%{app_name}
%config(noreplace) %{_sysconfdir}/udev/rules.d/99-windows-11-style-clipboard-history-input.rules
%{_datadir}/applications/io.github.mahdi-arts.clipboard-history.desktop
%{_datadir}/icons/hicolor/*/apps/io.github.mahdi-arts.clipboard-history.*
%{_datadir}/doc/%{name}/apparmor/%{app_name}

%changelog
* Thu Aug 21 2026 Mahdi Arts <mahdi-arts@users.noreply.github.com> - 2.5.0
- Initial RPM release
- Full feature parity with Debian package
- Support for Fedora, RHEL, and derivatives
