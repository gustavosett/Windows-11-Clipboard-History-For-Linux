use crate::session;
use std::thread;
use std::time::Duration;

type PasteStrategy = (&'static str, fn() -> Result<(), String>);

/// Delay before starting the paste sequence to ensure window focus is stable
const PRE_PASTE_DELAY_MS: u64 = 50;

/// Delay between key events to ensure proper registration
const KEY_EVENT_DELAY_MS: u64 = 50;

/// Delay after device creation for uinput to be recognized
const UINPUT_DEVICE_SETTLE_MS: u64 = 100;

/// Delay after paste sequence completes
const POST_PASTE_DELAY_MS: u64 = 30;

#[cfg(target_os = "linux")]
pub fn simulate_paste_keystroke() -> Result<(), String> {
    // Give window manager time to settle focus before sending keystrokes
    thread::sleep(Duration::from_millis(PRE_PASTE_DELAY_MS));

    eprintln!("[SimulatePaste] Sending Ctrl+V...");

    const X11_STRATEGIES: &[PasteStrategy] = &[
        ("xdotool", simulate_paste_xdotool),
        ("XTest", simulate_paste_xtest),
        ("uinput", simulate_paste_uinput),
    ];

    const NON_X11_STRATEGIES: &[PasteStrategy] = &[("uinput", simulate_paste_uinput)];

    let strategies = if session::is_x11() {
        X11_STRATEGIES
    } else {
        NON_X11_STRATEGIES
    };

    for (name, func) in strategies {
        match func() {
            Ok(()) => {
                eprintln!("[SimulatePaste] Ctrl+V sent via {}", name);
                // Small delay after paste to let the target app process it
                thread::sleep(Duration::from_millis(POST_PASTE_DELAY_MS));
                return Ok(());
            }
            Err(err) => {
                eprintln!("[SimulatePaste] {} failed: {}", name, err);
            }
        }
    }

    Err("All paste methods failed".to_string())
}

/// Helper for XTest input generation
#[cfg(target_os = "linux")]
fn fake_key<C: x11rb::connection::Connection + x11rb::protocol::xtest::ConnectionExt>(
    conn: &C,
    key_type: u8,
    keycode: u8,
    root_window: u32,
    ctx: &str,
) -> Result<(), String> {
    conn.xtest_fake_input(key_type, keycode, 0, root_window, 0, 0, 0)
        .map_err(|e| format!("{}: {}", ctx, e))?;
    conn.flush().map_err(|e| format!("Flush failed: {}", e))?;
    Ok(())
}

/// Simulate Ctrl+V using X11 XTest extension
#[cfg(target_os = "linux")]
fn simulate_paste_xtest() -> Result<(), String> {
    use x11rb::connection::Connection;
    use x11rb::protocol::xtest::ConnectionExt as XtestConnectionExt;
    use x11rb::wrapper::ConnectionExt as WrapperConnectionExt; // Imported for sync()

    const CTRL_L_KEYCODE: u8 = 37;
    const V_KEYCODE: u8 = 55;

    let (conn, screen_num) =
        x11rb::connect(None).map_err(|e| format!("X11 connect failed: {}", e))?;
    let screen = &conn.setup().roots[screen_num];
    let root_window = screen.root;

    conn.xtest_get_version(2, 1)
        .map_err(|e| format!("XTest version query failed: {}", e))?
        .reply()
        .map_err(|e| format!("XTest version query failed: {}", e))?;

    conn.sync()
        .map_err(|e| format!("Sync setup failed: {}", e))?;

    // Press Ctrl and wait for it to be registered
    fake_key(
        &conn,
        2,
        CTRL_L_KEYCODE,
        root_window,
        "Failed to press Ctrl",
    )?;
    conn.sync()
        .map_err(|e| format!("Sync after Ctrl press failed: {}", e))?;
    thread::sleep(Duration::from_millis(KEY_EVENT_DELAY_MS));

    // Press V
    fake_key(&conn, 2, V_KEYCODE, root_window, "Failed to press V")?;
    conn.sync()
        .map_err(|e| format!("Sync after V press failed: {}", e))?;
    thread::sleep(Duration::from_millis(KEY_EVENT_DELAY_MS));

    // Release V
    fake_key(&conn, 3, V_KEYCODE, root_window, "Failed to release V")?;
    conn.sync()
        .map_err(|e| format!("Sync after V release failed: {}", e))?;
    thread::sleep(Duration::from_millis(KEY_EVENT_DELAY_MS));

    // Release Ctrl
    fake_key(
        &conn,
        3,
        CTRL_L_KEYCODE,
        root_window,
        "Failed to release Ctrl",
    )?;

    conn.sync()
        .map_err(|e| format!("Final sync failed: {}", e))?;
    Ok(())
}

/// Simulate Ctrl+V using xdotool
#[cfg(target_os = "linux")]
fn simulate_paste_xdotool() -> Result<(), String> {
    // Send Ctrl+V to the currently focused window without specifying a target
    // Using --delay ensures proper timing between key events
    let output = std::process::Command::new("xdotool")
        .args(["key", "--delay"])
        .arg(KEY_EVENT_DELAY_MS.to_string())
        .arg("--clearmodifiers")
        .arg("ctrl+v")
        .output()
        .map_err(|e| format!("Failed to run xdotool key: {}", e))?;

    if output.status.success() {
        eprintln!("[SimulatePaste] xdotool sent ctrl+v to focused window");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("xdotool key failed: {}", stderr))
    }
}

#[cfg(target_os = "linux")]
fn simulate_paste_uinput() -> Result<(), String> {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::os::unix::io::AsRawFd;

    const EV_SYN: u16 = 0x00;
    const EV_KEY: u16 = 0x01;
    const SYN_REPORT: u16 = 0x00;
    const KEY_LEFTCTRL: u16 = 29;
    const KEY_V: u16 = 47;

    fn make_event(type_: u16, code: u16, value: i32) -> [u8; 24] {
        let mut event = [0u8; 24];
        event[16..18].copy_from_slice(&type_.to_ne_bytes());
        event[18..20].copy_from_slice(&code.to_ne_bytes());
        event[20..24].copy_from_slice(&value.to_ne_bytes());
        event
    }

    let mut uinput = OpenOptions::new()
        .write(true)
        .open("/dev/uinput")
        .map_err(|e| format!("Failed to open /dev/uinput: {}", e))?;

    const UI_SET_EVBIT: libc::c_ulong = 0x40045564;
    const UI_SET_KEYBIT: libc::c_ulong = 0x40045565;
    const UI_DEV_SETUP: libc::c_ulong = 0x405c5503;
    const UI_DEV_CREATE: libc::c_ulong = 0x5501;
    const UI_DEV_DESTROY: libc::c_ulong = 0x5502;

    unsafe {
        if libc::ioctl(uinput.as_raw_fd(), UI_SET_EVBIT, EV_KEY as libc::c_int) < 0 {
            return Err("Failed to set EV_KEY".to_string());
        }
        if libc::ioctl(
            uinput.as_raw_fd(),
            UI_SET_KEYBIT,
            KEY_LEFTCTRL as libc::c_int,
        ) < 0
        {
            return Err("Failed to set KEY_LEFTCTRL".to_string());
        }
        if libc::ioctl(uinput.as_raw_fd(), UI_SET_KEYBIT, KEY_V as libc::c_int) < 0 {
            return Err("Failed to set KEY_V".to_string());
        }

        #[repr(C)]
        struct UinputSetup {
            id: [u16; 4],
            name: [u8; 80],
            ff_effects_max: u32,
        }

        let mut setup = UinputSetup {
            id: [0x03, 0x1234, 0x5678, 0x0001],
            name: [0; 80],
            ff_effects_max: 0,
        };
        let name = b"emoji-paste-helper";
        setup.name[..name.len()].copy_from_slice(name);

        if libc::ioctl(uinput.as_raw_fd(), UI_DEV_SETUP, &setup) < 0 {
            return Err("Failed to setup uinput device".to_string());
        }
        if libc::ioctl(uinput.as_raw_fd(), UI_DEV_CREATE) < 0 {
            return Err("Failed to create uinput device".to_string());
        }
    }

    // Wait longer for the virtual device to be recognized by the system
    // This is critical for some desktop environments (Cinnamon, GNOME)
    thread::sleep(Duration::from_millis(UINPUT_DEVICE_SETTLE_MS));

    // Press Ctrl
    uinput
        .write_all(&make_event(EV_KEY, KEY_LEFTCTRL, 1))
        .map_err(|e| e.to_string())?;
    uinput
        .write_all(&make_event(EV_SYN, SYN_REPORT, 0))
        .map_err(|e| e.to_string())?;
    uinput.flush().map_err(|e| e.to_string())?;
    thread::sleep(Duration::from_millis(KEY_EVENT_DELAY_MS));

    // Press V
    uinput
        .write_all(&make_event(EV_KEY, KEY_V, 1))
        .map_err(|e| e.to_string())?;
    uinput
        .write_all(&make_event(EV_SYN, SYN_REPORT, 0))
        .map_err(|e| e.to_string())?;
    uinput.flush().map_err(|e| e.to_string())?;
    thread::sleep(Duration::from_millis(KEY_EVENT_DELAY_MS));

    // Release V
    uinput
        .write_all(&make_event(EV_KEY, KEY_V, 0))
        .map_err(|e| e.to_string())?;
    uinput
        .write_all(&make_event(EV_SYN, SYN_REPORT, 0))
        .map_err(|e| e.to_string())?;
    uinput.flush().map_err(|e| e.to_string())?;
    thread::sleep(Duration::from_millis(KEY_EVENT_DELAY_MS));

    // Release Ctrl
    uinput
        .write_all(&make_event(EV_KEY, KEY_LEFTCTRL, 0))
        .map_err(|e| e.to_string())?;
    uinput
        .write_all(&make_event(EV_SYN, SYN_REPORT, 0))
        .map_err(|e| e.to_string())?;
    uinput.flush().map_err(|e| e.to_string())?;

    // Wait for events to be processed before destroying device
    thread::sleep(Duration::from_millis(KEY_EVENT_DELAY_MS));

    unsafe {
        libc::ioctl(uinput.as_raw_fd(), UI_DEV_DESTROY);
    }

    // Small delay after device destruction
    thread::sleep(Duration::from_millis(POST_PASTE_DELAY_MS));

    Ok(())
}
