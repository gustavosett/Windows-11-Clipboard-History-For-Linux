use crate::session;

#[cfg(target_os = "linux")]
pub fn simulate_paste_keystroke() -> Result<(), String> {
    // Small delay before paste
    std::thread::sleep(std::time::Duration::from_millis(10));

    eprintln!("[SimulatePaste] Sending Ctrl+V...");

    // On X11, try multiple methods
    if session::is_x11() {
        // Try XTest first (fastest, no process spawn)
        if let Ok(()) = simulate_paste_xtest() {
            eprintln!("[SimulatePaste] Ctrl+V sent via XTest");
            return Ok(());
        }
        eprintln!("[SimulatePaste] XTest failed, trying xdotool...");

        // Try xdotool with window targeting
        if let Ok(()) = simulate_paste_xdotool() {
            eprintln!("[SimulatePaste] Ctrl+V sent via xdotool");
            return Ok(());
        }
        eprintln!("[SimulatePaste] xdotool failed, trying other fallbacks...");
    }

    // Try enigo (works on both X11 and Wayland)
    if let Ok(()) = simulate_paste_enigo() {
        eprintln!("[SimulatePaste] Ctrl+V sent via enigo");
        return Ok(());
    }

    // Fallback to uinput method
    if let Ok(()) = simulate_paste_uinput() {
        eprintln!("[SimulatePaste] Ctrl+V sent via uinput");
        return Ok(());
    }

    Err("All paste methods failed".to_string())
}

#[cfg(not(target_os = "linux"))]
pub fn simulate_paste_keystroke() -> Result<(), String> {
    Ok(())
}

/// Simulate Ctrl+V using xdotool with the focused window
#[cfg(target_os = "linux")]
fn simulate_paste_xdotool() -> Result<(), String> {
    // Get the currently focused window
    let window_output = std::process::Command::new("xdotool")
        .arg("getwindowfocus")
        .output()
        .map_err(|e| format!("Failed to run xdotool getwindowfocus: {}", e))?;

    if !window_output.status.success() {
        return Err("xdotool getwindowfocus failed".to_string());
    }

    let window_id = String::from_utf8_lossy(&window_output.stdout)
        .trim()
        .to_string();

    eprintln!("[SimulatePaste] xdotool targeting window: {}", window_id);

    // Send key to the specific window
    let output = std::process::Command::new("xdotool")
        .args(["key", "--window", &window_id, "--clearmodifiers", "ctrl+v"])
        .output()
        .map_err(|e| format!("Failed to run xdotool key: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("xdotool key failed: {}", stderr))
    }
}

/// Simulate Ctrl+V using X11 XTest extension
#[cfg(target_os = "linux")]
fn simulate_paste_xtest() -> Result<(), String> {
    use x11rb::connection::Connection;
    use x11rb::protocol::xtest::ConnectionExt as XtestConnectionExt;
    use x11rb::wrapper::ConnectionExt as WrapperConnectionExt;

    let (conn, screen_num) =
        x11rb::connect(None).map_err(|e| format!("X11 connect failed: {}", e))?;

    let screen = &conn.setup().roots[screen_num];
    let root_window = screen.root;

    // Get keycode for Control_L and V
    let ctrl_keycode = 37u8; // Control_L
    let v_keycode = 55u8; // V

    // Verify XTest extension is available
    conn.xtest_get_version(2, 1)
        .map_err(|e| format!("XTest not available: {}", e))?
        .reply()
        .map_err(|e| format!("XTest version query failed: {}", e))?;

    conn.sync()
        .map_err(|e| format!("Sync setup failed: {}", e))?;

    // Press Ctrl (type=2 is KeyPress)
    conn.xtest_fake_input(2, ctrl_keycode, 0, root_window, 0, 0, 0)
        .map_err(|e| format!("Failed to press Ctrl: {}", e))?;
    conn.flush().map_err(|e| format!("Flush failed: {}", e))?;

    // Small delay between key events
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Press V
    conn.xtest_fake_input(2, v_keycode, 0, root_window, 0, 0, 0)
        .map_err(|e| format!("Failed to press V: {}", e))?;
    conn.flush().map_err(|e| format!("Flush failed: {}", e))?;

    std::thread::sleep(std::time::Duration::from_millis(10));

    // Release V (type=3 is KeyRelease)
    conn.xtest_fake_input(3, v_keycode, 0, root_window, 0, 0, 0)
        .map_err(|e| format!("Failed to release V: {}", e))?;
    conn.flush().map_err(|e| format!("Flush failed: {}", e))?;

    std::thread::sleep(std::time::Duration::from_millis(5));

    // Release Ctrl
    conn.xtest_fake_input(3, ctrl_keycode, 0, root_window, 0, 0, 0)
        .map_err(|e| format!("Failed to release Ctrl: {}", e))?;
    conn.flush().map_err(|e| format!("Flush failed: {}", e))?;

    // Sync to ensure all events are processed
    conn.sync().map_err(|e| format!("Sync failed: {}", e))?;

    Ok(())
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

    std::thread::sleep(std::time::Duration::from_millis(50));

    // Press Ctrl
    uinput
        .write_all(&make_event(EV_KEY, KEY_LEFTCTRL, 1))
        .map_err(|e| e.to_string())?;
    uinput
        .write_all(&make_event(EV_SYN, SYN_REPORT, 0))
        .map_err(|e| e.to_string())?;
    uinput.flush().map_err(|e| e.to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Press V
    uinput
        .write_all(&make_event(EV_KEY, KEY_V, 1))
        .map_err(|e| e.to_string())?;
    uinput
        .write_all(&make_event(EV_SYN, SYN_REPORT, 0))
        .map_err(|e| e.to_string())?;
    uinput.flush().map_err(|e| e.to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Release V
    uinput
        .write_all(&make_event(EV_KEY, KEY_V, 0))
        .map_err(|e| e.to_string())?;
    uinput
        .write_all(&make_event(EV_SYN, SYN_REPORT, 0))
        .map_err(|e| e.to_string())?;
    uinput.flush().map_err(|e| e.to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Release Ctrl
    uinput
        .write_all(&make_event(EV_KEY, KEY_LEFTCTRL, 0))
        .map_err(|e| e.to_string())?;
    uinput
        .write_all(&make_event(EV_SYN, SYN_REPORT, 0))
        .map_err(|e| e.to_string())?;
    uinput.flush().map_err(|e| e.to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(50));

    unsafe {
        libc::ioctl(uinput.as_raw_fd(), UI_DEV_DESTROY);
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn simulate_paste_enigo() -> Result<(), String> {
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};

    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;

    enigo
        .key(Key::Control, Direction::Press)
        .map_err(|e| e.to_string())?;
    enigo
        .key(Key::Unicode('v'), Direction::Click)
        .map_err(|e| e.to_string())?;
    enigo
        .key(Key::Control, Direction::Release)
        .map_err(|e| e.to_string())?;

    Ok(())
}
