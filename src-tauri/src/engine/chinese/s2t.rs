use std::collections::HashMap;
use lazy_static::lazy_static;
use windows_sys::Win32::System::DataExchange::{
    OpenClipboard, CloseClipboard, EmptyClipboard, GetClipboardData, SetClipboardData,
};
use windows_sys::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GHND};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_CONTROL,
};
use crate::engine::hook::input_sender::VCKEY_EXTRA_INFO;

lazy_static! {
    pub static ref S2T_MAP: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::with_capacity(10500);
        let raw = include_str!("s2t_dict.txt");
        for line in raw.lines() {
            let mut parts = line.split('\t');
            if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                m.insert(k, v);
            }
        }
        m
    };

    pub static ref T2S_MAP: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::with_capacity(9500);
        let raw = include_str!("t2s_dict.txt");
        for line in raw.lines() {
            let mut parts = line.split('\t');
            if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                m.insert(k, v);
            }
        }
        m
    };
}

/// Converts Simplified Chinese to Traditional Chinese using greedy longest-match.
pub fn convert_s2t(text: &str) -> String {
    convert_with_map(text, &S2T_MAP)
}

/// Converts Traditional Chinese to Simplified Chinese using greedy longest-match.
pub fn convert_t2s(text: &str) -> String {
    convert_with_map(text, &T2S_MAP)
}

fn convert_with_map(text: &str, map: &HashMap<&'static str, &'static str>) -> String {
    if text.is_empty() {
        return String::new();
    }
    if let Some(&t) = map.get(text) {
        return t.to_string();
    }

    let mut result = String::with_capacity(text.len() * 2);
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let n = chars.len();

    while i < n {
        let mut matched = false;
        let max_len = 16.min(n - i);
        for l in (1..=max_len).rev() {
            let sub: String = chars[i..i + l].iter().collect();
            if let Some(&t) = map.get(sub.as_str()) {
                result.push_str(t);
                i += l;
                matched = true;
                break;
            }
        }
        if !matched {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

pub enum ConversionResult {
    ToTraditional(String),
    ToSimplified(String),
}

/// Intelligently detects whether the text contains more Traditional or Simplified characters,
/// and converts it to the opposite form.
pub fn convert_smart(text: &str) -> ConversionResult {
    let s2t = convert_s2t(text);
    let t2s = convert_t2s(text);

    let s2t_diff = text.chars().zip(s2t.chars()).filter(|(a, b)| a != b).count()
        + if text.len() != s2t.len() { 1 } else { 0 };
    let t2s_diff = text.chars().zip(t2s.chars()).filter(|(a, b)| a != b).count()
        + if text.len() != t2s.len() { 1 } else { 0 };

    if t2s_diff > s2t_diff {
        // Text is mostly Traditional -> Convert to Simplified
        ConversionResult::ToSimplified(t2s)
    } else {
        // Text is mostly Simplified -> Convert to Traditional
        ConversionResult::ToTraditional(s2t)
    }
}

pub fn get_clipboard_text() -> Option<String> {
    unsafe {
        if OpenClipboard(0 as _) == 0 {
            return None;
        }
        const CF_UNICODETEXT: u32 = 13;
        let handle = GetClipboardData(CF_UNICODETEXT);
        if handle.is_null() {
            CloseClipboard();
            return None;
        }
        let ptr = GlobalLock(handle) as *const u16;
        if ptr.is_null() {
            CloseClipboard();
            return None;
        }
        let mut len = 0;
        while *ptr.add(len) != 0 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(ptr, len);
        let s = String::from_utf16_lossy(slice);
        GlobalUnlock(handle);
        CloseClipboard();
        Some(s)
    }
}

pub fn set_clipboard_text(text: &str) -> bool {
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let bytes_len = wide.len() * 2;
    unsafe {
        if OpenClipboard(0 as _) == 0 {
            return false;
        }
        EmptyClipboard();
        let handle = GlobalAlloc(GHND, bytes_len);
        if handle.is_null() {
            CloseClipboard();
            return false;
        }
        let ptr = GlobalLock(handle) as *mut u16;
        if ptr.is_null() {
            CloseClipboard();
            return false;
        }
        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
        GlobalUnlock(handle);
        const CF_UNICODETEXT: u32 = 13;
        SetClipboardData(CF_UNICODETEXT, handle);
        CloseClipboard();
        true
    }
}

fn simulate_ctrl_c() {
    let mut inputs: [INPUT; 4] = unsafe { std::mem::zeroed() };

    inputs[0].r#type = INPUT_KEYBOARD;
    inputs[0].Anonymous.ki = KEYBDINPUT {
        wVk: VK_CONTROL,
        wScan: 0x1D,
        dwFlags: 0,
        time: 0,
        dwExtraInfo: VCKEY_EXTRA_INFO,
    };

    inputs[1].r#type = INPUT_KEYBOARD;
    inputs[1].Anonymous.ki = KEYBDINPUT {
        wVk: 0x43, // 'C'
        wScan: 0x2E,
        dwFlags: 0,
        time: 0,
        dwExtraInfo: VCKEY_EXTRA_INFO,
    };

    inputs[2].r#type = INPUT_KEYBOARD;
    inputs[2].Anonymous.ki = KEYBDINPUT {
        wVk: 0x43,
        wScan: 0x2E,
        dwFlags: KEYEVENTF_KEYUP,
        time: 0,
        dwExtraInfo: VCKEY_EXTRA_INFO,
    };

    inputs[3].r#type = INPUT_KEYBOARD;
    inputs[3].Anonymous.ki = KEYBDINPUT {
        wVk: VK_CONTROL,
        wScan: 0x1D,
        dwFlags: KEYEVENTF_KEYUP,
        time: 0,
        dwExtraInfo: VCKEY_EXTRA_INFO,
    };

    unsafe {
        SendInput(4, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
    }
}

fn simulate_ctrl_v() {
    let mut inputs: [INPUT; 4] = unsafe { std::mem::zeroed() };

    inputs[0].r#type = INPUT_KEYBOARD;
    inputs[0].Anonymous.ki = KEYBDINPUT {
        wVk: VK_CONTROL,
        wScan: 0x1D,
        dwFlags: 0,
        time: 0,
        dwExtraInfo: VCKEY_EXTRA_INFO,
    };

    inputs[1].r#type = INPUT_KEYBOARD;
    inputs[1].Anonymous.ki = KEYBDINPUT {
        wVk: 0x56, // 'V'
        wScan: 0x2F,
        dwFlags: 0,
        time: 0,
        dwExtraInfo: VCKEY_EXTRA_INFO,
    };

    inputs[2].r#type = INPUT_KEYBOARD;
    inputs[2].Anonymous.ki = KEYBDINPUT {
        wVk: 0x56,
        wScan: 0x2F,
        dwFlags: KEYEVENTF_KEYUP,
        time: 0,
        dwExtraInfo: VCKEY_EXTRA_INFO,
    };

    inputs[3].r#type = INPUT_KEYBOARD;
    inputs[3].Anonymous.ki = KEYBDINPUT {
        wVk: VK_CONTROL,
        wScan: 0x1D,
        dwFlags: KEYEVENTF_KEYUP,
        time: 0,
        dwExtraInfo: VCKEY_EXTRA_INFO,
    };

    unsafe {
        SendInput(4, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
    }
}

/// Automatically copies selected text via Ctrl+C, converts between Simplified & Traditional Chinese,
/// sets the converted text into the clipboard, and pastes it in place via Ctrl+V.
pub fn convert_selected_text_inplace() {
    std::thread::spawn(|| {
        // 1. Simulate Ctrl+C to copy selected text
        simulate_ctrl_c();

        // 2. Wait 70ms for the target application to update the clipboard
        std::thread::sleep(std::time::Duration::from_millis(70));

        // 3. Read clipboard text
        if let Some(text) = get_clipboard_text() {
            if text.trim().is_empty() {
                return;
            }

            // Check if text has any CJK characters (0x4E00..=0x9FFF, 0x3400..=0x4DBF, etc.)
            let has_cjk = text.chars().any(|c| {
                let u = c as u32;
                (0x4E00..=0x9FFF).contains(&u) || (0x3400..=0x4DBF).contains(&u) || (0x20000..=0x2CEAF).contains(&u)
            });

            if !has_cjk {
                return;
            }

            // 4. Smart bidirectional conversion
            let (converted_text, msg) = match convert_smart(&text) {
                ConversionResult::ToTraditional(t) => {
                    (t, "VCKeys: ĐÃ CHUYỂN SANG PHỒN THỂ (繁體)")
                }
                ConversionResult::ToSimplified(s) => {
                    (s, "VCKeys: ĐÃ CHUYỂN SANG GIẢN THỂ (简体)")
                }
            };

            // 5. Update clipboard with converted text
            set_clipboard_text(&converted_text);

            // 6. Simulate Ctrl+V to replace the highlighted selection with converted text
            simulate_ctrl_v();

            // 7. Show native OSD Toast
            crate::engine::osd::show_osd_custom("CN", msg, 0x00EB6325);
        }
    });
}
