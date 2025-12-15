//! Global Hotkey Manager Module
//! Handles global keyboard shortcuts using evdev for direct input device access
//! This works across X11, Wayland, and even TTY - truly global hotkeys

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

#[cfg(target_os = "linux")]
use evdev::{Device, InputEventKind, Key};

/// Actions triggered by hotkeys
#[derive(Debug, Clone, Copy)]
pub enum HotkeyAction {
    Toggle,
    Close,
}

/// Manages global hotkey listening
pub struct HotkeyManager {
    running: Arc<AtomicBool>,
    _handles: Vec<JoinHandle<()>>,
}

impl HotkeyManager {
    /// Create a new hotkey manager with a callback for when the hotkey is pressed
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(HotkeyAction) + Send + Sync + 'static,
    {
        let running = Arc::new(AtomicBool::new(true));
        let callback = Arc::new(callback);

        #[cfg(target_os = "linux")]
        let handles = Self::start_evdev_listeners(running.clone(), callback);

        #[cfg(not(target_os = "linux"))]
        let handles = Vec::new();

        Self {
            running,
            _handles: handles,
        }
    }

    #[cfg(target_os = "linux")]
    fn start_evdev_listeners<F>(running: Arc<AtomicBool>, callback: Arc<F>) -> Vec<JoinHandle<()>>
    where
        F: Fn(HotkeyAction) + Send + Sync + 'static,
    {
        eprintln!("[HotkeyManager] Starting evdev-based global hotkey listener...");

        // Find all keyboard devices
        let keyboards = Self::find_keyboard_devices();

        if keyboards.is_empty() {
            eprintln!("[HotkeyManager] ERROR: No keyboard devices found!");
            eprintln!(
                "[HotkeyManager] Make sure user is in 'input' group: sudo usermod -aG input $USER"
            );
            return Vec::new();
        }

        eprintln!(
            "[HotkeyManager] Found {} keyboard device(s)",
            keyboards.len()
        );

        // Shared state for modifier keys
        let super_pressed = Arc::new(AtomicBool::new(false));
        let ctrl_pressed = Arc::new(AtomicBool::new(false));
        let alt_pressed = Arc::new(AtomicBool::new(false));

        let mut handles = Vec::new();

        for device_path in keyboards {
            let running = running.clone();
            let callback = callback.clone();
            let super_pressed = super_pressed.clone();
            let ctrl_pressed = ctrl_pressed.clone();
            let alt_pressed = alt_pressed.clone();

            let handle = thread::spawn(move || {
                if let Err(e) = Self::listen_device(
                    &device_path,
                    running,
                    callback,
                    super_pressed,
                    ctrl_pressed,
                    alt_pressed,
                ) {
                    eprintln!("[HotkeyManager] Error listening to {}: {}", device_path, e);
                }
            });

            handles.push(handle);
        }

        handles
    }

    #[cfg(target_os = "linux")]
    fn find_keyboard_devices() -> Vec<String> {
        let mut keyboards = Vec::new();

        // Try to enumerate devices from /dev/input/
        if let Ok(entries) = std::fs::read_dir("/dev/input") {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    if name_str.starts_with("event") {
                        // Try to open and check if it's a keyboard
                        if let Ok(device) = Device::open(&path) {
                            // Check if device has keyboard keys
                            if let Some(keys) = device.supported_keys() {
                                // Check for common keyboard keys
                                if keys.contains(Key::KEY_A)
                                    && keys.contains(Key::KEY_LEFTCTRL)
                                    && keys.contains(Key::KEY_LEFTMETA)
                                {
                                    let path_str = path.to_string_lossy().to_string();
                                    eprintln!(
                                        "[HotkeyManager] Found keyboard: {} ({})",
                                        path_str,
                                        device.name().unwrap_or("Unknown")
                                    );
                                    keyboards.push(path_str);
                                }
                            }
                        }
                    }
                }
            }
        }

        keyboards
    }

    #[cfg(target_os = "linux")]
    fn listen_device<F>(
        device_path: &str,
        running: Arc<AtomicBool>,
        callback: Arc<F>,
        super_pressed: Arc<AtomicBool>,
        ctrl_pressed: Arc<AtomicBool>,
        alt_pressed: Arc<AtomicBool>,
    ) -> Result<(), String>
    where
        F: Fn(HotkeyAction) + Send + Sync + 'static,
    {
        let mut device = Device::open(device_path).map_err(|e| e.to_string())?;

        eprintln!("[HotkeyManager] Listening on: {}", device_path);

        loop {
            if !running.load(Ordering::SeqCst) {
                break;
            }

            // Fetch events with a timeout to allow checking running flag
            match device.fetch_events() {
                Ok(events) => {
                    for event in events {
                        if let InputEventKind::Key(key) = event.kind() {
                            let pressed = event.value() == 1; // 1 = pressed, 0 = released, 2 = repeat
                            let released = event.value() == 0;

                            match key {
                                Key::KEY_LEFTMETA | Key::KEY_RIGHTMETA => {
                                    if pressed {
                                        super_pressed.store(true, Ordering::SeqCst);
                                    } else if released {
                                        super_pressed.store(false, Ordering::SeqCst);
                                    }
                                }
                                Key::KEY_LEFTCTRL | Key::KEY_RIGHTCTRL => {
                                    if pressed {
                                        ctrl_pressed.store(true, Ordering::SeqCst);
                                    } else if released {
                                        ctrl_pressed.store(false, Ordering::SeqCst);
                                    }
                                }
                                Key::KEY_LEFTALT | Key::KEY_RIGHTALT => {
                                    if pressed {
                                        alt_pressed.store(true, Ordering::SeqCst);
                                    } else if released {
                                        alt_pressed.store(false, Ordering::SeqCst);
                                    }
                                }
                                Key::KEY_V => {
                                    if pressed {
                                        let super_down = super_pressed.load(Ordering::SeqCst);
                                        let ctrl_down = ctrl_pressed.load(Ordering::SeqCst);
                                        let alt_down = alt_pressed.load(Ordering::SeqCst);

                                        // Super+V or Ctrl+Alt+V
                                        if super_down || (ctrl_down && alt_down) {
                                            eprintln!("[HotkeyManager] Toggle hotkey triggered!");
                                            callback(HotkeyAction::Toggle);
                                        }
                                    }
                                }
                                Key::KEY_ESC => {
                                    if pressed {
                                        eprintln!("[HotkeyManager] ESC pressed");
                                        callback(HotkeyAction::Close);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Err(e) => {
                    // EAGAIN is expected when no events are available
                    if e.raw_os_error() != Some(11) {
                        eprintln!("[HotkeyManager] Error reading events: {}", e);
                        // Small delay before retrying
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
            }
        }

        Ok(())
    }

    /// Stop the hotkey listener
    #[allow(dead_code)]
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        self.stop();
    }
}
