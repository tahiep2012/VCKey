use std::mem::size_of;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VK_BACK, VK_DELETE
};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetClassNameW};

pub const VCKEY_EXTRA_INFO: usize = 0xFE0000FE;

/// Detects if the current active/foreground window belongs to Microsoft Excel.
pub fn is_active_window_excel() -> bool {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return false;
        }
        let mut class_name: [u16; 64] = [0; 64];
        let len = GetClassNameW(hwnd, class_name.as_mut_ptr(), 64);
        if len > 0 {
            let class_str = String::from_utf16_lossy(&class_name[..len as usize]);
            if class_str.contains("XLMAIN") || class_str.contains("EXCEL") {
                return true;
            }
        }
        false
    }
}

pub fn send_unicode_string(s: &str) {
    if s.is_empty() {
        return;
    }
    send_replacement(0, s);
}

pub fn send_backspaces(count: usize) {
    if count == 0 {
        return;
    }
    send_replacement(count, "");
}

/// Atomically deletes `backspaces` characters and types `append_str` synchronously.
/// Uses OEM Scan Code `0x0E` for Backspace.
/// When in Microsoft Excel, it dismisses Excel's forward AutoComplete ghost suggestion first so Backspaces
/// operate cleanly on typed text. For all other apps (Outlook, Word, Notepad, Chrome), VK_DELETE is never sent.
pub fn send_replacement(backspaces: usize, append_str: &str) {
    if backspaces == 0 && append_str.is_empty() {
        return;
    }

    let is_excel = is_active_window_excel();
    let utf16_units: Vec<u16> = append_str.encode_utf16().collect();
    let total_inputs = (if is_excel && backspaces > 0 { 2 } else { 0 }) + backspaces * 2 + utf16_units.len() * 2;
    let mut inputs: Vec<INPUT> = Vec::with_capacity(total_inputs);

    // 0. ONLY for Microsoft Excel: dismiss AutoComplete ghost forward selection
    // In Excel cell editing, the ghost suggestion is forward-selected. Sending VK_DELETE in Excel
    // dismisses the ghost selection so Backspaces delete the actual typed characters.
    // For Outlook, Word, Notepad, browsers, etc., this is NEVER executed.
    if is_excel && backspaces > 0 {
        let mut del_down: INPUT = unsafe { std::mem::zeroed() };
        del_down.r#type = INPUT_KEYBOARD;
        del_down.Anonymous.ki = KEYBDINPUT {
            wVk: VK_DELETE,
            wScan: 0x53,
            dwFlags: KEYEVENTF_EXTENDEDKEY,
            time: 0,
            dwExtraInfo: VCKEY_EXTRA_INFO,
        };
        inputs.push(del_down);

        let mut del_up: INPUT = unsafe { std::mem::zeroed() };
        del_up.r#type = INPUT_KEYBOARD;
        del_up.Anonymous.ki = KEYBDINPUT {
            wVk: VK_DELETE,
            wScan: 0x53,
            dwFlags: KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP,
            time: 0,
            dwExtraInfo: VCKEY_EXTRA_INFO,
        };
        inputs.push(del_up);
    }

    // 1. Backspaces with standard OEM scan code 0x0E
    for _ in 0..backspaces {
        // Key Down
        let mut input_down: INPUT = unsafe { std::mem::zeroed() };
        input_down.r#type = INPUT_KEYBOARD;
        input_down.Anonymous.ki = KEYBDINPUT {
            wVk: VK_BACK,
            wScan: 0x0E,
            dwFlags: 0,
            time: 0,
            dwExtraInfo: VCKEY_EXTRA_INFO,
        };
        inputs.push(input_down);

        // Key Up
        let mut input_up: INPUT = unsafe { std::mem::zeroed() };
        input_up.r#type = INPUT_KEYBOARD;
        input_up.Anonymous.ki = KEYBDINPUT {
            wVk: VK_BACK,
            wScan: 0x0E,
            dwFlags: KEYEVENTF_KEYUP,
            time: 0,
            dwExtraInfo: VCKEY_EXTRA_INFO,
        };
        inputs.push(input_up);
    }

    // 2. UTF-16 Unicode Characters
    for ch in utf16_units {
        // Down
        let mut input_down: INPUT = unsafe { std::mem::zeroed() };
        input_down.r#type = INPUT_KEYBOARD;
        input_down.Anonymous.ki = KEYBDINPUT {
            wVk: 0,
            wScan: ch,
            dwFlags: KEYEVENTF_UNICODE,
            time: 0,
            dwExtraInfo: VCKEY_EXTRA_INFO,
        };
        inputs.push(input_down);

        // Up
        let mut input_up: INPUT = unsafe { std::mem::zeroed() };
        input_up.r#type = INPUT_KEYBOARD;
        input_up.Anonymous.ki = KEYBDINPUT {
            wVk: 0,
            wScan: ch,
            dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
            time: 0,
            dwExtraInfo: VCKEY_EXTRA_INFO,
        };
        inputs.push(input_up);
    }

    unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_mut_ptr(),
            size_of::<INPUT>() as i32,
        );
    }
}

