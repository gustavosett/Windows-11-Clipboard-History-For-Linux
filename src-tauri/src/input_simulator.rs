use crate::session;
use parking_lot::Mutex;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

type PasteStrategy = (&'static str, fn() -> Result<(), String>);

/// The kernel creates a uinput device synchronously, but udev/libinput and the
/// compositor discover it asynchronously. Device readiness is paid once at
/// startup, never for every paste.
const DEVICE_READY_TIMEOUT: Duration = Duration::from_secs(2);
const DEVICE_READY_POLL_INTERVAL: Duration = Duration::from_millis(5);
const DEVICE_DISCOVERY_GRACE: Duration = Duration::from_millis(100);

/// X11 key-state checks are round trips to the X server. They normally finish
/// immediately; the timeout is only a failure ceiling for a stalled server.
const X11_KEY_STATE_TIMEOUT: Duration = Duration::from_millis(250);
const X11_KEY_STATE_POLL_INTERVAL: Duration = Duration::from_millis(1);

/// Kept only for the last-resort xdotool fallback. The primary XTest path
/// verifies the actual key state and needs no fixed inter-key delay.
const XDOTOOL_KEY_DELAY_MS: &str = "50";

const EV_SYN: u16 = 0x00;
const EV_KEY: u16 = 0x01;
const SYN_REPORT: u16 = 0x00;
const KEY_LEFTCTRL: u16 = 29;
const KEY_V: u16 = 47;

const UI_SET_EVBIT: libc::c_ulong = 0x40045564;
const UI_SET_KEYBIT: libc::c_ulong = 0x40045565;
const UI_DEV_SETUP: libc::c_ulong = 0x405c5503;
const UI_DEV_CREATE: libc::c_ulong = 0x5501;
const UI_DEV_DESTROY: libc::c_ulong = 0x5502;

const X11_KEY_PRESS: u8 = 2;
const X11_KEY_RELEASE: u8 = 3;
const XK_CONTROL_L: u32 = 0xffe3;
const XK_CONTROL_R: u32 = 0xffe4;
const XK_LOWER_V: u32 = 0x0076;
const XK_UPPER_V: u32 = 0x0056;

/// Persistent virtual keyboard. Recreating it for every paste allowed the
/// compositor to attach after Ctrl but before V, which produced a literal
/// `v` under load.
struct UinputDevice {
    file: File,
}

/// Persistent XTest client. Connection setup, extension negotiation and
/// keyboard-map discovery happen once rather than on every paste.
struct XtestDevice {
    connection: x11rb::rust_connection::RustConnection,
    root_window: u32,
    ctrl_keycode: u8,
    v_keycode: u8,
}

static UINPUT_DEVICE: OnceLock<Mutex<Option<UinputDevice>>> = OnceLock::new();
static XTEST_DEVICE: OnceLock<Mutex<Option<XtestDevice>>> = OnceLock::new();

/// Warms the active input backend outside the paste hot path. If startup
/// initialization fails, the first paste retries it.
pub fn init() {
    let (thread_name, warmup): (&str, fn()) = if session::is_x11() {
        ("xtest-paste-warmup", warm_up_xtest)
    } else {
        ("uinput-paste-warmup", warm_up_uinput)
    };

    if let Err(error) = std::thread::Builder::new()
        .name(thread_name.to_string())
        .spawn(warmup)
    {
        eprintln!("[SimulatePaste] Failed to start warmup thread: {}", error);
    }
}

fn warm_up_xtest() {
    let mut device = xtest_device_lock().lock();
    if device.is_some() {
        return;
    }

    match XtestDevice::create() {
        Ok(created) => {
            *device = Some(created);
            eprintln!("[XTest] Persistent X11 input client is ready");
        }
        Err(error) => eprintln!(
            "[XTest] Startup warmup failed; first paste will retry: {}",
            error
        ),
    }
}

fn warm_up_uinput() {
    let mut device = uinput_device_lock().lock();
    if device.is_some() {
        return;
    }

    match UinputDevice::create() {
        Ok(created) => {
            *device = Some(created);
            eprintln!("[uinput] Virtual keyboard warmed up at startup");
        }
        Err(error) => eprintln!(
            "[uinput] Startup warmup failed; first paste will retry: {}",
            error
        ),
    }
}

pub fn simulate_paste_keystroke() -> Result<(), String> {
    eprintln!("[SimulatePaste] Sending Ctrl+V...");

    // XTest uses one native X11 connection and verifies that Ctrl is down
    // before V. xdotool is retained only as a compatibility fallback.
    const X11_STRATEGIES: &[PasteStrategy] = &[
        ("XTest", simulate_paste_xtest),
        ("xdotool", simulate_paste_xdotool),
        ("uinput", simulate_paste_uinput),
    ];
    const NON_X11_STRATEGIES: &[PasteStrategy] = &[("uinput", simulate_paste_uinput)];

    let strategies = if session::is_x11() {
        X11_STRATEGIES
    } else {
        NON_X11_STRATEGIES
    };

    for (name, strategy) in strategies {
        match strategy() {
            Ok(()) => {
                eprintln!("[SimulatePaste] Ctrl+V sent via {}", name);
                return Ok(());
            }
            Err(error) => eprintln!("[SimulatePaste] {} failed: {}", name, error),
        }
    }

    Err("All paste methods failed".to_string())
}

// =============================================================================
// X11 / XTest
// =============================================================================

fn fake_key<C: x11rb::connection::Connection + x11rb::protocol::xtest::ConnectionExt>(
    conn: &C,
    key_type: u8,
    keycode: u8,
    root_window: u32,
    context: &str,
) -> Result<(), String> {
    conn.xtest_fake_input(key_type, keycode, 0, root_window, 0, 0, 0)
        .map_err(|error| format!("{}: {}", context, error))?;
    conn.flush()
        .map_err(|error| format!("X11 flush failed: {}", error))
}

fn simulate_paste_xtest() -> Result<(), String> {
    let mut device = xtest_device_lock().lock();
    if device.is_none() {
        *device = Some(XtestDevice::create()?);
    }

    let result = device
        .as_mut()
        .expect("XTest device was initialized")
        .send_ctrl_v();
    if result.is_err() {
        // Reconnect on the next attempt if the X server connection went away.
        *device = None;
    }
    result
}

fn xtest_device_lock() -> &'static Mutex<Option<XtestDevice>> {
    XTEST_DEVICE.get_or_init(|| Mutex::new(None))
}

impl XtestDevice {
    fn create() -> Result<Self, String> {
        use x11rb::connection::Connection;
        use x11rb::protocol::xtest::ConnectionExt as XtestConnectionExt;

        let (connection, screen_num) =
            x11rb::connect(None).map_err(|error| format!("X11 connect failed: {}", error))?;
        let root_window = connection
            .setup()
            .roots
            .get(screen_num)
            .ok_or("X11 screen not found")?
            .root;

        connection
            .xtest_get_version(2, 1)
            .map_err(|error| format!("XTest version query failed: {}", error))?
            .reply()
            .map_err(|error| format!("XTest version query failed: {}", error))?;

        let (ctrl_keycode, v_keycode) = resolve_paste_keycodes(&connection)?;

        Ok(Self {
            connection,
            root_window,
            ctrl_keycode,
            v_keycode,
        })
    }

    fn send_ctrl_v(&mut self) -> Result<(), String> {
        let mut ctrl_down = false;
        let mut v_down = false;

        let operation = (|| {
            fake_key(
                &self.connection,
                X11_KEY_PRESS,
                self.ctrl_keycode,
                self.root_window,
                "Failed to press Ctrl",
            )?;
            ctrl_down = true;
            wait_for_x11_key_state(&self.connection, self.ctrl_keycode, true)?;

            fake_key(
                &self.connection,
                X11_KEY_PRESS,
                self.v_keycode,
                self.root_window,
                "Failed to press V",
            )?;
            v_down = true;
            wait_for_x11_key_state(&self.connection, self.v_keycode, true)?;

            fake_key(
                &self.connection,
                X11_KEY_RELEASE,
                self.v_keycode,
                self.root_window,
                "Failed to release V",
            )?;
            v_down = false;

            fake_key(
                &self.connection,
                X11_KEY_RELEASE,
                self.ctrl_keycode,
                self.root_window,
                "Failed to release Ctrl",
            )?;
            ctrl_down = false;

            wait_for_x11_key_state(&self.connection, self.v_keycode, false)?;
            wait_for_x11_key_state(&self.connection, self.ctrl_keycode, false)
        })();

        // Never leave a synthetic modifier held if a round trip fails midway.
        let mut cleanup_errors = Vec::new();
        if v_down {
            if let Err(error) = fake_key(
                &self.connection,
                X11_KEY_RELEASE,
                self.v_keycode,
                self.root_window,
                "Cleanup failed to release V",
            ) {
                cleanup_errors.push(error);
            }
        }
        if ctrl_down {
            if let Err(error) = fake_key(
                &self.connection,
                X11_KEY_RELEASE,
                self.ctrl_keycode,
                self.root_window,
                "Cleanup failed to release Ctrl",
            ) {
                cleanup_errors.push(error);
            }
        }

        match (operation, cleanup_errors.is_empty()) {
            (Ok(()), true) => Ok(()),
            (Ok(()), false) => Err(cleanup_errors.join("; ")),
            (Err(error), true) => Err(error),
            (Err(error), false) => Err(format!("{}; {}", error, cleanup_errors.join("; "))),
        }
    }
}

fn resolve_paste_keycodes<C: x11rb::connection::Connection>(conn: &C) -> Result<(u8, u8), String> {
    use x11rb::protocol::xproto::ConnectionExt as XprotoConnectionExt;

    let setup = conn.setup();
    let first_keycode = setup.min_keycode;
    let keycode_count = u16::from(setup.max_keycode)
        .checked_sub(u16::from(first_keycode))
        .and_then(|count| count.checked_add(1))
        .and_then(|count| u8::try_from(count).ok())
        .ok_or("Invalid X11 keycode range")?;

    let mapping = conn
        .get_keyboard_mapping(first_keycode, keycode_count)
        .map_err(|error| format!("Failed to query X11 keymap: {}", error))?
        .reply()
        .map_err(|error| format!("Failed to query X11 keymap: {}", error))?;

    let ctrl = find_keycode(
        &mapping.keysyms,
        mapping.keysyms_per_keycode,
        first_keycode,
        &[XK_CONTROL_L, XK_CONTROL_R],
    )
    .ok_or("Ctrl key not found in the active X11 keymap")?;
    let v = find_keycode(
        &mapping.keysyms,
        mapping.keysyms_per_keycode,
        first_keycode,
        &[XK_LOWER_V, XK_UPPER_V],
    )
    .ok_or("V key not found in the active X11 keymap")?;

    Ok((ctrl, v))
}

fn find_keycode(
    keysyms: &[u32],
    keysyms_per_keycode: u8,
    first_keycode: u8,
    wanted: &[u32],
) -> Option<u8> {
    let width = usize::from(keysyms_per_keycode);
    if width == 0 {
        return None;
    }

    keysyms
        .chunks(width)
        .position(|symbols| symbols.iter().any(|symbol| wanted.contains(symbol)))
        .and_then(|offset| u8::try_from(offset).ok())
        .and_then(|offset| first_keycode.checked_add(offset))
}

fn wait_for_x11_key_state<C: x11rb::connection::Connection>(
    conn: &C,
    keycode: u8,
    expected_pressed: bool,
) -> Result<(), String> {
    use x11rb::protocol::xproto::ConnectionExt as XprotoConnectionExt;

    let start = Instant::now();
    loop {
        let reply = conn
            .query_keymap()
            .map_err(|error| format!("Failed to query X11 key state: {}", error))?
            .reply()
            .map_err(|error| format!("Failed to query X11 key state: {}", error))?;

        if key_is_pressed(&reply.keys, keycode) == expected_pressed {
            return Ok(());
        }
        if start.elapsed() >= X11_KEY_STATE_TIMEOUT {
            return Err(format!(
                "Timed out waiting for X11 key {} to become {}",
                keycode,
                if expected_pressed {
                    "pressed"
                } else {
                    "released"
                }
            ));
        }
        std::thread::sleep(X11_KEY_STATE_POLL_INTERVAL);
    }
}

fn key_is_pressed(keymap: &[u8; 32], keycode: u8) -> bool {
    let byte = usize::from(keycode / 8);
    let bit = keycode % 8;
    keymap[byte] & (1 << bit) != 0
}

fn simulate_paste_xdotool() -> Result<(), String> {
    let output = std::process::Command::new("xdotool")
        .args([
            "key",
            "--delay",
            XDOTOOL_KEY_DELAY_MS,
            "--clearmodifiers",
            "ctrl+v",
        ])
        .output()
        .map_err(|error| format!("Failed to run xdotool: {}", error))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "xdotool failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

// =============================================================================
// Wayland / uinput
// =============================================================================

fn uinput_device_lock() -> &'static Mutex<Option<UinputDevice>> {
    UINPUT_DEVICE.get_or_init(|| Mutex::new(None))
}

impl UinputDevice {
    fn create() -> Result<Self, String> {
        let file = OpenOptions::new()
            .write(true)
            .open("/dev/uinput")
            .map_err(|error| format!("Failed to open /dev/uinput: {}", error))?;
        let fd = file.as_raw_fd();

        unsafe {
            if libc::ioctl(fd, UI_SET_EVBIT, EV_KEY as libc::c_int) < 0 {
                return Err(last_os_error("Failed to enable EV_KEY"));
            }
            if libc::ioctl(fd, UI_SET_KEYBIT, KEY_LEFTCTRL as libc::c_int) < 0 {
                return Err(last_os_error("Failed to enable KEY_LEFTCTRL"));
            }
            if libc::ioctl(fd, UI_SET_KEYBIT, KEY_V as libc::c_int) < 0 {
                return Err(last_os_error("Failed to enable KEY_V"));
            }
        }

        #[repr(C)]
        struct UinputSetup {
            id: libc::input_id,
            name: [u8; 80],
            ff_effects_max: u32,
        }

        let device_name = format!("win11-clipboard-paste-{}", std::process::id());
        let mut setup = UinputSetup {
            id: libc::input_id {
                bustype: 0x03,
                vendor: 0x1234,
                product: 0x5678,
                version: 0x0001,
            },
            name: [0; 80],
            ff_effects_max: 0,
        };
        setup.name[..device_name.len()].copy_from_slice(device_name.as_bytes());

        unsafe {
            if libc::ioctl(fd, UI_DEV_SETUP, &setup) < 0 {
                return Err(last_os_error("Failed to configure uinput device"));
            }
            if libc::ioctl(fd, UI_DEV_CREATE) < 0 {
                return Err(last_os_error("Failed to create uinput device"));
            }
        }

        if let Err(error) = wait_for_uinput_device(&device_name) {
            unsafe {
                libc::ioctl(fd, UI_DEV_DESTROY);
            }
            return Err(error);
        }

        // The event node is an observable udev barrier. This one-time grace is
        // outside the hot path after startup and lets the compositor attach.
        std::thread::sleep(DEVICE_DISCOVERY_GRACE);
        eprintln!("[uinput] Persistent virtual keyboard is ready");

        Ok(Self { file })
    }

    fn send_ctrl_v(&mut self) -> Result<(), String> {
        let sequence = ctrl_v_sequence();
        if let Err(error) = self.write_events(&sequence) {
            let _ = self.release_all_keys();
            return Err(error);
        }
        Ok(())
    }

    fn release_all_keys(&mut self) -> Result<(), String> {
        self.write_events(&[
            input_event(EV_KEY, KEY_V, 0),
            input_event(EV_KEY, KEY_LEFTCTRL, 0),
            input_event(EV_SYN, SYN_REPORT, 0),
        ])
    }

    fn write_events(&mut self, events: &[libc::input_event]) -> Result<(), String> {
        self.file
            .write_all(input_events_as_bytes(events))
            .map_err(|error| format!("Failed to write uinput sequence: {}", error))
    }
}

impl Drop for UinputDevice {
    fn drop(&mut self) {
        let _ = self.release_all_keys();
        unsafe {
            libc::ioctl(self.file.as_raw_fd(), UI_DEV_DESTROY);
        }
    }
}

fn simulate_paste_uinput() -> Result<(), String> {
    let mut device = uinput_device_lock().lock();

    if let Some(existing) = device.as_mut() {
        match existing.send_ctrl_v() {
            Ok(()) => return Ok(()),
            Err(error) => {
                eprintln!("[uinput] Existing device failed; recreating: {}", error);
                *device = None;
            }
        }
    }

    let mut created = UinputDevice::create()?;
    created.send_ctrl_v()?;
    *device = Some(created);
    Ok(())
}

fn ctrl_v_sequence() -> [libc::input_event; 6] {
    [
        // One frame makes Ctrl and V visible together; the second releases
        // both. There is no scheduling window between separate key writes.
        input_event(EV_KEY, KEY_LEFTCTRL, 1),
        input_event(EV_KEY, KEY_V, 1),
        input_event(EV_SYN, SYN_REPORT, 0),
        input_event(EV_KEY, KEY_V, 0),
        input_event(EV_KEY, KEY_LEFTCTRL, 0),
        input_event(EV_SYN, SYN_REPORT, 0),
    ]
}

fn input_event(type_: u16, code: u16, value: i32) -> libc::input_event {
    // input_event has target-specific time fields. Zeroing the libc type uses
    // the correct ABI on both 32- and 64-bit Linux, unlike a hardcoded 24-byte
    // buffer.
    let mut event: libc::input_event = unsafe { std::mem::zeroed() };
    event.type_ = type_;
    event.code = code;
    event.value = value;
    event
}

fn input_events_as_bytes(events: &[libc::input_event]) -> &[u8] {
    // SAFETY: input_event is a plain C ABI struct and the returned byte slice
    // has exactly the lifetime and initialized size of the source slice.
    unsafe {
        std::slice::from_raw_parts(events.as_ptr().cast::<u8>(), std::mem::size_of_val(events))
    }
}

fn wait_for_uinput_device(device_name: &str) -> Result<PathBuf, String> {
    let start = Instant::now();
    loop {
        if let Some(path) = find_uinput_event_node(device_name) {
            eprintln!(
                "[uinput] Event node {} appeared after {:?}",
                path.display(),
                start.elapsed()
            );
            return Ok(path);
        }
        if start.elapsed() >= DEVICE_READY_TIMEOUT {
            return Err(format!(
                "Timed out waiting for '{}' to appear in /dev/input",
                device_name
            ));
        }
        std::thread::sleep(DEVICE_READY_POLL_INTERVAL);
    }
}

fn find_uinput_event_node(device_name: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir("/sys/class/input").ok()?;
    for entry in entries.flatten() {
        let name = match std::fs::read_to_string(entry.path().join("name")) {
            Ok(name) => name,
            Err(_) => continue,
        };
        if name.trim() != device_name {
            continue;
        }

        let children = match std::fs::read_dir(entry.path()) {
            Ok(children) => children,
            Err(_) => continue,
        };
        for child in children.flatten() {
            let file_name = child.file_name();
            let file_name = file_name.to_string_lossy();
            if file_name.starts_with("event") {
                let path = Path::new("/dev/input").join(file_name.as_ref());
                if path.exists() {
                    return Some(path);
                }
            }
        }
    }
    None
}

fn last_os_error(context: &str) -> String {
    format!("{}: {}", context, std::io::Error::last_os_error())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctrl_v_is_emitted_as_two_complete_frames() {
        let events = ctrl_v_sequence();
        let actual: Vec<(u16, u16, i32)> = events
            .iter()
            .map(|event| (event.type_, event.code, event.value))
            .collect();

        assert_eq!(
            actual,
            vec![
                (EV_KEY, KEY_LEFTCTRL, 1),
                (EV_KEY, KEY_V, 1),
                (EV_SYN, SYN_REPORT, 0),
                (EV_KEY, KEY_V, 0),
                (EV_KEY, KEY_LEFTCTRL, 0),
                (EV_SYN, SYN_REPORT, 0),
            ]
        );
    }

    #[test]
    fn input_event_bytes_use_the_platform_abi_size() {
        let events = ctrl_v_sequence();
        assert_eq!(
            input_events_as_bytes(&events).len(),
            events.len() * std::mem::size_of::<libc::input_event>()
        );
    }

    #[test]
    fn keymap_bit_lookup_handles_keycode_boundaries() {
        let mut keymap = [0u8; 32];
        keymap[4] = 1 << 5; // keycode 37
        keymap[6] = 1 << 7; // keycode 55

        assert!(key_is_pressed(&keymap, 37));
        assert!(key_is_pressed(&keymap, 55));
        assert!(!key_is_pressed(&keymap, 36));
        assert!(!key_is_pressed(&keymap, 56));
    }

    #[test]
    fn keycodes_are_resolved_from_the_active_mapping() {
        // Three keycodes starting at 20, with two symbols each.
        let mapping = [0, 0, XK_CONTROL_L, 0, XK_LOWER_V, XK_UPPER_V];

        assert_eq!(
            find_keycode(&mapping, 2, 20, &[XK_CONTROL_L, XK_CONTROL_R]),
            Some(21)
        );
        assert_eq!(
            find_keycode(&mapping, 2, 20, &[XK_LOWER_V, XK_UPPER_V]),
            Some(22)
        );
    }
}
