use std::sync::Arc;
use parking_lot::Mutex;
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetForegroundWindow, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
    KBDLLHOOKSTRUCT, MSLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WH_MOUSE_LL,
    WM_KEYDOWN, WM_SYSKEYDOWN,
    WM_LBUTTONDOWN, WM_RBUTTONDOWN, WM_MBUTTONDOWN,
    WM_NCLBUTTONDOWN, WM_NCRBUTTONDOWN, WM_NCMBUTTONDOWN,
    WindowFromPoint,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyState, VK_CAPITAL, VK_CONTROL, VK_MENU, VK_SHIFT,
    VK_BACK, VK_ESCAPE, VK_RETURN, VK_SPACE, VK_TAB,
    VK_LEFT, VK_UP, VK_RIGHT, VK_DOWN, VK_HOME, VK_END, VK_PRIOR, VK_NEXT, VK_DELETE,
};

use crate::engine::vietnamese::VietnameseTelexEngine;
use crate::engine::chinese::{ChineseEngine, Candidate, UserDict, convert_s2t, show_candidate_popup, hide_candidate_popup};
use crate::engine::hook::input_sender::{send_replacement, send_unicode_string, VCKEY_EXTRA_INFO};

#[derive(Clone, Debug)]
pub struct WordEntry {
    pub raw: String,
    pub displayed: String,
    pub spaces_after: usize,
}

pub struct HookState {
    pub mode: String,               // "Vietnamese", "Chinese", "English"
    pub variant: String,            // "Simplified", "Traditional"
    pub vn_raw_word: String,
    pub vn_displayed_str: String,
    pub vn_word_history: Vec<WordEntry>,
    pub cn_raw_buffer: String,
    pub cn_candidates: Vec<Candidate>,
    pub current_page: usize,
    pub page_size: usize,
    pub font_size: usize,
    pub ctrl_shift_cancelled: bool,
    pub alt_shift_cancelled: bool,
    pub last_foreground_hwnd: isize,
    // Theo dõi cụm từ ghép liên tiếp để học từ điển cá nhân
    pub cn_prev_committed_text: String,   // chữ Hán vừa commit gần nhất
    pub cn_prev_committed_pinyin: String, // pinyin tương ứng với commit gần nhất
    pub chinese_punctuation: bool,
    pub chinese_quote_open: bool,
    pub chinese_single_quote_open: bool,
    // Smart-typing flags (mirrored from AppSettings, updated on save)
    pub vn_auto_english_restore: bool,
    // Modifier tracking flags (to prevent GetAsyncKeyState ghost/stuck states)
    pub ctrl_down: bool,
    pub alt_down: bool,
    pub win_down: bool,
    pub shift_down: bool,
}

impl HookState {
    pub fn reset_vietnamese_buffers(&mut self) {
        self.vn_raw_word.clear();
        self.vn_displayed_str.clear();
        self.vn_word_history.clear();
    }

    pub fn reset_modifiers(&mut self) {
        self.ctrl_down = false;
        self.alt_down = false;
        self.win_down = false;
        self.shift_down = false;
    }
}

impl Default for HookState {
    fn default() -> Self {
        Self {
            mode: "Vietnamese".to_string(),
            variant: "Simplified".to_string(),
            vn_raw_word: String::new(),
            vn_displayed_str: String::new(),
            vn_word_history: Vec::new(),
            cn_raw_buffer: String::new(),
            cn_candidates: Vec::new(),
            current_page: 0,
            page_size: 5,
            font_size: 17,
            ctrl_shift_cancelled: false,
            alt_shift_cancelled: false,
            last_foreground_hwnd: 0,
            cn_prev_committed_text: String::new(),
            cn_prev_committed_pinyin: String::new(),
            chinese_punctuation: true,
            chinese_quote_open: false,
            chinese_single_quote_open: false,
            vn_auto_english_restore: true,
            ctrl_down: false,
            alt_down: false,
            win_down: false,
            shift_down: false,
        }
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_STATE: Arc<Mutex<HookState>> = Arc::new(Mutex::new(HookState::default()));
    pub static ref CHINESE_ENGINE: ChineseEngine = ChineseEngine::new();
    pub static ref USER_DICT: Arc<Mutex<UserDict>> = {
        // Đặt file cạnh thư mục chạy exe (dist/config/vckey_user_dict.json)
        let dict_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("config")
            .join("vckey_user_dict.json");
        Arc::new(Mutex::new(UserDict::load(dict_path)))
    };
}

pub unsafe extern "system" fn mouse_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code >= 0 {
        let msg = w_param as u32;
        if msg == WM_LBUTTONDOWN
            || msg == WM_RBUTTONDOWN
            || msg == WM_MBUTTONDOWN
            || msg == WM_NCLBUTTONDOWN
            || msg == WM_NCRBUTTONDOWN
            || msg == WM_NCMBUTTONDOWN
        {
            let cand_hwnd = crate::engine::chinese::candidate_window::CAND_HWND.load(std::sync::atomic::Ordering::Relaxed);
            let mouse = *(l_param as *const MSLLHOOKSTRUCT);
            let target_hwnd = WindowFromPoint(mouse.pt) as isize;

            // If user clicks anywhere outside candidate window, reset Vietnamese buffer & modifiers
            if cand_hwnd == 0 || target_hwnd != cand_hwnd {
                let mut state = GLOBAL_STATE.lock();
                state.reset_vietnamese_buffers();
                state.reset_modifiers();
                if !state.cn_raw_buffer.is_empty() {
                    state.cn_raw_buffer.clear();
                    state.cn_candidates.clear();
                    state.current_page = 0;
                    drop(state);
                    hide_candidate_popup();
                }
            }
        }
    }
    CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param)
}

pub unsafe extern "system" fn keyboard_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code < 0 {
        return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
    }

    let kbd = *(l_param as *const KBDLLHOOKSTRUCT);
    let vk_code = kbd.vkCode as u16;
    let extra_info = kbd.dwExtraInfo as usize;

    // Ignore self-injected keystrokes
    if extra_info == VCKEY_EXTRA_INFO {
        return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
    }

    let mut state = GLOBAL_STATE.lock();

    // Focus change check
    let curr_hwnd = GetForegroundWindow() as isize;
    let cand_hwnd = crate::engine::chinese::candidate_window::CAND_HWND.load(std::sync::atomic::Ordering::Relaxed);
    if curr_hwnd != 0 && curr_hwnd != cand_hwnd && curr_hwnd != state.last_foreground_hwnd {
        state.last_foreground_hwnd = curr_hwnd;
        state.reset_vietnamese_buffers();
        state.reset_modifiers();
        state.cn_raw_buffer.clear();
        state.cn_candidates.clear();
        state.cn_prev_committed_text.clear();
        state.cn_prev_committed_pinyin.clear();
        hide_candidate_popup();
    }

    let is_down = w_param as u32 == WM_KEYDOWN || w_param as u32 == WM_SYSKEYDOWN;

    let is_ctrl = vk_code == VK_CONTROL || vk_code == 0xA2 || vk_code == 0xA3;
    let is_shift = vk_code == VK_SHIFT || vk_code == 0xA0 || vk_code == 0xA1;
    let is_alt = vk_code == VK_MENU || vk_code == 0xA4 || vk_code == 0xA5;
    let is_win = vk_code == 0x5B || vk_code == 0x5C;

    if is_ctrl {
        state.ctrl_down = is_down;
    }
    if is_shift {
        state.shift_down = is_down;
    }
    if is_alt {
        state.alt_down = is_down;
    }
    if is_win {
        state.win_down = is_down;
    }

    // Double validation: combine hook-tracked state with GetAsyncKeyState to permanently eliminate ghost stuck keys
    let ctrl_pressed = (state.ctrl_down || (is_ctrl && is_down)) && ((GetAsyncKeyState(VK_CONTROL as i32) as u16 & 0x8000) != 0 || is_ctrl);
    let shift_pressed = state.shift_down || (is_shift && is_down) || ((GetAsyncKeyState(VK_SHIFT as i32) as u16 & 0x8000) != 0);
    let alt_pressed = (state.alt_down || (is_alt && is_down)) && ((GetAsyncKeyState(VK_MENU as i32) as u16 & 0x8000) != 0 || is_alt);
    let win_pressed = (state.win_down || (is_win && is_down)) && ((GetAsyncKeyState(0x5B) as u16 & 0x8000) != 0 || (GetAsyncKeyState(0x5C) as u16 & 0x8000) != 0 || is_win);

    // 1. Hotkey: Ctrl + Shift + F -> Toggle Simplified / Traditional Chinese typing mode
    if ctrl_pressed && shift_pressed && !alt_pressed && (vk_code == 0x46 || vk_code == 0x66) && is_down {
        state.ctrl_shift_cancelled = true;
        drop(state);
        crate::toggle_chinese_variant();
        return 1; // Handled! Swallow 'F' key
    }

    // 1b. Hotkey: In-place text conversion for selected Chinese text (Ctrl+Alt+F OR Ctrl+Shift+F9)
    if ((ctrl_pressed && alt_pressed && (vk_code == 0x46 || vk_code == 0x66))
        || (ctrl_pressed && shift_pressed && (vk_code == 0x77 || vk_code == 0x78))) // F8 (0x77) or F9 (0x78)
        && is_down
    {
        state.ctrl_shift_cancelled = true;
        state.alt_shift_cancelled = true;
        drop(state);
        crate::engine::chinese::s2t::convert_selected_text_inplace();
        return 1; // Handled! Swallow the trigger key
    }

    // Cancel chord hotkeys if any non-modifier key is pressed (e.g. Ctrl+Shift+F cancels Ctrl+Shift toggle)
    if is_down && !is_ctrl && !is_shift && !is_alt && !is_win && vk_code != VK_CAPITAL {
        if ctrl_pressed && shift_pressed {
            state.ctrl_shift_cancelled = true;
        }
        if alt_pressed && shift_pressed {
            state.alt_shift_cancelled = true;
        }
    }

    // Reset cancellation flags when modifiers are released
    if (!ctrl_pressed || !shift_pressed) && !is_down {
        state.ctrl_shift_cancelled = false;
    }
    if (!alt_pressed || !shift_pressed) && !is_down {
        state.alt_shift_cancelled = false;
    }

    // Modifier keys passthrough without clearing word buffer
    if is_ctrl || is_shift || is_alt || is_win || vk_code == VK_CAPITAL {
        return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
    }

    // System shortcuts passthrough (Ctrl+C, Ctrl+V, Ctrl+A, Ctrl+Z, Alt+Tab, Win+..., etc.)
    if ctrl_pressed || alt_pressed || win_pressed {
        state.reset_vietnamese_buffers();
        state.cn_raw_buffer.clear();
        state.cn_candidates.clear();
        hide_candidate_popup();
        return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
    }

    if !is_down {
        return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
    }

    // --- A. ENGLISH MODE ---
    if state.mode == "English" {
        return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
    }

    // --- B. VIETNAMESE TELEX MODE ---
    if state.mode == "Vietnamese" {
        // Space key: push current word into history stack, track consecutive spaces
        if vk_code == VK_SPACE {
            if !state.vn_raw_word.is_empty() {
                let entry = WordEntry {
                    raw: state.vn_raw_word.clone(),
                    displayed: state.vn_displayed_str.clone(),
                    spaces_after: 1,
                };
                state.vn_word_history.push(entry);
                if state.vn_word_history.len() > 30 {
                    state.vn_word_history.remove(0);
                }
                state.vn_raw_word.clear();
                state.vn_displayed_str.clear();
            } else if let Some(last) = state.vn_word_history.last_mut() {
                last.spaces_after += 1;
            }
            return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
        }

        // Return / Escape / Tab / Navigation keys: word break & clear history
        if vk_code == VK_RETURN
            || vk_code == VK_ESCAPE
            || vk_code == VK_TAB
            || vk_code == VK_LEFT
            || vk_code == VK_RIGHT
            || vk_code == VK_UP
            || vk_code == VK_DOWN
            || vk_code == VK_HOME
            || vk_code == VK_END
            || vk_code == VK_PRIOR
            || vk_code == VK_NEXT
            || vk_code == VK_DELETE
        {
            state.reset_vietnamese_buffers();
            return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
        }

        // Backspace: handle rollback within word and cross-word backspace restoration
        if vk_code == VK_BACK {
            if !state.vn_raw_word.is_empty() {
                let curr_len = state.vn_displayed_str.chars().count();
                let target_len = curr_len.saturating_sub(1);
                while !state.vn_raw_word.is_empty()
                    && VietnameseTelexEngine::transform(&state.vn_raw_word).chars().count() > target_len
                {
                    state.vn_raw_word.pop();
                }
                state.vn_displayed_str = VietnameseTelexEngine::transform(&state.vn_raw_word);
                return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
            } else if let Some(last) = state.vn_word_history.last_mut() {
                if last.spaces_after > 1 {
                    last.spaces_after -= 1;
                    return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
                } else {
                    // last.spaces_after == 1
                    // This Backspace deletes the separating space on screen in Windows.
                    // The cursor moves directly to the end of the previous word.
                    // We restore the previous word's raw keystrokes and displayed string so subsequent
                    // tone keys (s, f, r, x, j) or vowel modifiers (w, a, e, o) modify the restored word!
                    let entry = state.vn_word_history.pop().unwrap();
                    state.vn_raw_word = entry.raw;
                    state.vn_displayed_str = entry.displayed;
                    return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
                }
            }
            return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
        }

        let current_im = crate::engine::vietnamese::unikey_ffi::UnikeyEngine::get_input_method();
        use crate::engine::vietnamese::unikey_ffi::InputMethod;

        let mut key_char: Option<char> = None;

        // 1. Standard letters A-Z (Uppercase & Lowercase with Shift and CapsLock support)
        if (0x41..=0x5A).contains(&vk_code) {
            let caps_on = (GetKeyState(VK_CAPITAL as i32) & 0x0001) != 0;
            let is_upper = caps_on ^ shift_pressed;
            let raw_ch = (vk_code as u8) as char;
            key_char = Some(if is_upper {
                raw_ch.to_ascii_uppercase()
            } else {
                raw_ch.to_ascii_lowercase()
            });
        }
        // 2. Numbers 0-9 in VNI mode
        else if current_im == InputMethod::Vni && !shift_pressed && ((0x30..=0x39).contains(&vk_code) || (0x60..=0x69).contains(&vk_code)) {
            let digit_char = if (0x30..=0x39).contains(&vk_code) {
                ((vk_code - 0x30) as u8 + b'0') as char
            } else {
                ((vk_code - 0x60) as u8 + b'0') as char
            };
            if !state.vn_raw_word.is_empty() || digit_char == '9' {
                key_char = Some(digit_char);
            }
        }
        // 3. VIQR symbols: ' (0xDE), ` (0xC0), ? (0xBF with Shift), ~ (0xC0 with Shift), . (0xBE), ^ (0x36 with Shift), + (0xBB with Shift), * (0x38 with Shift), ( (0x39 with Shift), \ (0xDC), 0 (0x30)
        else if current_im == InputMethod::Viqr && !state.vn_raw_word.is_empty() {
            match vk_code {
                0xDE if !shift_pressed => key_char = Some('\''),
                0xC0 if !shift_pressed => key_char = Some('`'),
                0xBF if shift_pressed => key_char = Some('?'),
                0xC0 if shift_pressed => key_char = Some('~'),
                0xBE if !shift_pressed => key_char = Some('.'),
                0x36 if shift_pressed => key_char = Some('^'),
                0xBB if shift_pressed => key_char = Some('+'),
                0x38 if shift_pressed => key_char = Some('*'),
                0x39 if shift_pressed => key_char = Some('('),
                0xDC if !shift_pressed => key_char = Some('\\'),
                0x30 if !shift_pressed => key_char = Some('0'),
                _ => {}
            }
        }
        // 4. Bracket keys [ ] for Telex
        else if current_im == InputMethod::Telex {
            if vk_code == 0xDB && !shift_pressed {
                key_char = Some('[');
            } else if vk_code == 0xDD && !shift_pressed {
                key_char = Some(']');
            } else if vk_code == 0xDB && shift_pressed {
                key_char = Some('{');
            } else if vk_code == 0xDD && shift_pressed {
                key_char = Some('}');
            }
        }

        if let Some(ch) = key_char {
            // Self-healing: if buffer reaches 15 chars, it cannot be a single valid Vietnamese syllable.
            // Reset buffer to prevent corrupted/stale buffer from permanently blocking diacritics.
            if state.vn_raw_word.chars().count() >= 15 {
                state.reset_vietnamese_buffers();
            }
            state.vn_raw_word.push(ch);

            let transformed = VietnameseTelexEngine::transform(&state.vn_raw_word);
            let old_chars: Vec<char> = state.vn_displayed_str.chars().collect();
            let new_chars: Vec<char> = transformed.chars().collect();

            // Calculate common prefix to avoid deleting and re-typing identical characters (Anti-flicker)
            let mut common_prefix_len = 0;
            while common_prefix_len < old_chars.len()
                && common_prefix_len < new_chars.len()
                && old_chars[common_prefix_len] == new_chars[common_prefix_len]
            {
                common_prefix_len += 1;
            }

            let backspaces_needed = old_chars.len() - common_prefix_len;
            let suffix_to_send: String = new_chars[common_prefix_len..].iter().collect();

            // If no backspaces needed and suffix matches the raw typed key, let the hardware event pass through
            // naturally to Windows so shortcuts (like Shift+F, single letter tools in apps) fire properly!
            if backspaces_needed == 0 && suffix_to_send == ch.to_string() {
                state.vn_displayed_str = transformed;
                return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
            }

            send_replacement(backspaces_needed, &suffix_to_send);
            state.vn_displayed_str = transformed;
            return 1; // Handled
        }

        // Other keys (numbers, symbols, punctuation)
        state.reset_vietnamese_buffers();
        return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
    }

    // --- C. CHINESE PINYIN MODE ---
    if state.mode == "Chinese" {
        // A-Z letter keys (Pinyin input)
        if (0x41..=0x5A).contains(&vk_code) {
            let ch = ((vk_code as u8) as char).to_ascii_lowercase();
            state.cn_raw_buffer.push(ch);
            state.current_page = 0;
            {
                let user_dict = USER_DICT.lock();
                state.cn_candidates = CHINESE_ENGINE.lookup_with_user_dict(&state.cn_raw_buffer, &user_dict);
            }
            show_candidate_popup(
                &state.cn_raw_buffer,
                &state.cn_candidates,
                state.current_page,
                state.page_size,
                &state.variant,
            );
            return 1; // Handled
        }

        // Single quote ' (syllable separator 0xDE)
        if vk_code == 0xDE && !state.cn_raw_buffer.is_empty() {
            state.cn_raw_buffer.push('\'');
            {
                let user_dict = USER_DICT.lock();
                state.cn_candidates = CHINESE_ENGINE.lookup_with_user_dict(&state.cn_raw_buffer, &user_dict);
            }
            show_candidate_popup(
                &state.cn_raw_buffer,
                &state.cn_candidates,
                state.current_page,
                state.page_size,
                &state.variant,
            );
            return 1;
        }

        let is_v_mode = state.cn_raw_buffer.starts_with('v') || state.cn_raw_buffer.starts_with('V');

        // V-Mode: Special character inputs (digits 0-9, dot, math operators +, -, *, /, parentheses)
        if is_v_mode {
            let v_char: Option<char> = match vk_code {
                // Digits 0-9 (Main keyboard without Shift)
                0x30..=0x39 if !shift_pressed && !ctrl_pressed => {
                    Some(((vk_code - 0x30) as u8 + b'0') as char)
                }
                // Digits 0-9 (Numpad)
                0x60..=0x69 => {
                    Some(((vk_code - 0x60) as u8 + b'0') as char)
                }
                // Dot . (Main keyboard 0xBE without Shift, or Numpad 0x6E)
                0xBE if !shift_pressed => Some('.'),
                0x6E => Some('.'),
                // Math + (Shift + = 0xBB, or Numpad + 0x6B)
                0xBB if shift_pressed => Some('+'),
                0x6B => Some('+'),
                // Math - (0xBD without Shift, or Numpad - 0x6D)
                0xBD if !shift_pressed => Some('-'),
                0x6D => Some('-'),
                // Math * (Shift + 8 0x38, or Numpad * 0x6A)
                0x38 if shift_pressed => Some('*'),
                0x6A => Some('*'),
                // Math / (0xBF without Shift, or Numpad / 0x6F)
                0xBF if !shift_pressed => Some('/'),
                0x6F => Some('/'),
                // Parentheses ( (Shift + 9) and ) (Shift + 0)
                0x39 if shift_pressed => Some('('),
                0x30 if shift_pressed => Some(')'),
                _ => None,
            };

            if let Some(ch) = v_char {
                state.cn_raw_buffer.push(ch);
                state.current_page = 0;
                {
                    let user_dict = USER_DICT.lock();
                    state.cn_candidates = CHINESE_ENGINE.lookup_with_user_dict(&state.cn_raw_buffer, &user_dict);
                }
                show_candidate_popup(
                    &state.cn_raw_buffer,
                    &state.cn_candidates,
                    state.current_page,
                    state.page_size,
                    &state.variant,
                );
                return 1; // Handled
            }

            // In V-Mode: Ctrl + 1..9 directly selects candidate 1..9
            if ctrl_pressed && (0x31..=0x39).contains(&vk_code) {
                let num = (vk_code - 0x30) as usize;
                let index = state.current_page * state.page_size + (num - 1);
                if index < state.cn_candidates.len() {
                    let candidate = &state.cn_candidates[index];
                    let commit_text = if state.variant == "Traditional" {
                        convert_s2t(&candidate.text)
                    } else {
                        candidate.text.clone()
                    };
                    send_unicode_string(&commit_text);
                    state.cn_raw_buffer.clear();
                    state.cn_candidates.clear();
                    state.current_page = 0;
                    hide_candidate_popup();
                    return 1;
                }
            }
        }

        // Number keys 1-9 (Candidate selection for Normal Pinyin Mode)
        if !is_v_mode && !state.cn_raw_buffer.is_empty() && (0x31..=0x39).contains(&vk_code) && !shift_pressed {
            let num = (vk_code - 0x30) as usize;
            let index = state.current_page * state.page_size + (num - 1);
            if index < state.cn_candidates.len() {
                let candidate = &state.cn_candidates[index];
                let raw_candidate_text = candidate.text.clone();
                let commit_text = if state.variant == "Traditional" {
                    convert_s2t(&raw_candidate_text)
                } else {
                    raw_candidate_text.clone()
                };
                let raw_pinyin = state.cn_raw_buffer.clone();
                let clean_pinyin = raw_pinyin.replace('\'', "").to_lowercase();

                // 1. Luôn lưu lựa chọn của người dùng cho cụm pinyin hiện tại vào User Dict!
                {
                    let mut user_dict = USER_DICT.lock();
                    user_dict.add_phrase(&clean_pinyin, &raw_candidate_text);

                    // 2. Học cụm từ liên tiếp (chain): nếu trước đó cũng vừa commit tiếng Trung, ghép lại và lưu
                    if !state.cn_prev_committed_text.is_empty() {
                        let combined_text = format!("{}{}", state.cn_prev_committed_text, raw_candidate_text);
                        let combined_pinyin = format!("{}{}", state.cn_prev_committed_pinyin, clean_pinyin);
                        user_dict.add_phrase(&combined_pinyin, &combined_text);
                    }
                    user_dict.save();
                }

                // Cập nhật prev commit
                state.cn_prev_committed_text = raw_candidate_text.clone();
                state.cn_prev_committed_pinyin = clean_pinyin;

                send_unicode_string(&commit_text);
                state.cn_raw_buffer.clear();
                state.cn_candidates.clear();
                state.current_page = 0;
                hide_candidate_popup();
                return 1;
            }
        }

        // Chinese Punctuation Auto-conversion (，。、《》！？：；“”‘’……——【】（）～)
        if state.chinese_punctuation {
            let cn_punct = match vk_code {
                // , or <
                0xBC if !shift_pressed => Some("，"),
                0xBC if shift_pressed => Some("《"),
                // . or >
                0xBE if !shift_pressed => Some("。"),
                0xBE if shift_pressed => Some("》"),
                // / or ?
                0xBF if shift_pressed => Some("？"),
                // \ or |
                0xDC if !shift_pressed => Some("、"),
                // ; or :
                0xBA if !shift_pressed => Some("；"),
                0xBA if shift_pressed => Some("："),
                // ! (Shift + 1)
                0x31 if shift_pressed => Some("！"),
                // ^ (Shift + 6) -> ……
                0x36 if shift_pressed => Some("……"),
                // ( and ) (Shift + 9, Shift + 0)
                0x39 if shift_pressed => Some("（"),
                0x30 if shift_pressed => Some("）"),
                // [ and ] (when buffer is empty, so as not to conflict with candidate page navigation)
                0xDB if !shift_pressed && state.cn_raw_buffer.is_empty() => Some("【"),
                0xDD if !shift_pressed && state.cn_raw_buffer.is_empty() => Some("】"),
                // _ (Shift + -) -> ——
                0xBD if shift_pressed => Some("——"),
                // ~ (Shift + `)
                0xC0 if shift_pressed => Some("～"),
                // " (Shift + ') -> smart quotes “”
                0xDE if shift_pressed => {
                    let q = if state.chinese_quote_open { "”" } else { "“" };
                    state.chinese_quote_open = !state.chinese_quote_open;
                    Some(q)
                },
                // ' (without Shift) -> smart quotes ‘’ (only when cn_raw_buffer is empty)
                0xDE if !shift_pressed && state.cn_raw_buffer.is_empty() => {
                    let q = if state.chinese_single_quote_open { "’" } else { "‘" };
                    state.chinese_single_quote_open = !state.chinese_single_quote_open;
                    Some(q)
                },
                _ => None,
            };

            if let Some(punct_str) = cn_punct {
                // If there's an active pinyin buffer, commit candidate #1 first!
                if !state.cn_raw_buffer.is_empty() {
                    let index = state.current_page * state.page_size;
                    if index < state.cn_candidates.len() {
                        let candidate = &state.cn_candidates[index];
                        let commit_text = if state.variant == "Traditional" {
                            convert_s2t(&candidate.text)
                        } else {
                            candidate.text.clone()
                        };
                        send_unicode_string(&commit_text);
                    } else {
                        send_unicode_string(&state.cn_raw_buffer);
                    }
                    state.cn_raw_buffer.clear();
                    state.cn_candidates.clear();
                    state.current_page = 0;
                    state.cn_prev_committed_text.clear();
                    state.cn_prev_committed_pinyin.clear();
                    hide_candidate_popup();
                }

                send_unicode_string(punct_str);
                return 1; // Handled!
            }
        }

        // Space key: Select 1st candidate or commit raw
        if vk_code == VK_SPACE && !state.cn_raw_buffer.is_empty() {
            let index = state.current_page * state.page_size;
            let raw_pinyin = state.cn_raw_buffer.clone();
            let clean_pinyin = raw_pinyin.replace('\'', "").to_lowercase();
            if index < state.cn_candidates.len() {
                let candidate = &state.cn_candidates[index];
                let raw_candidate_text = candidate.text.clone();
                let commit_text = if state.variant == "Traditional" {
                    convert_s2t(&raw_candidate_text)
                } else {
                    raw_candidate_text.clone()
                };

                // 1. Luôn lưu lựa chọn của người dùng cho cụm pinyin hiện tại vào User Dict!
                {
                    let mut user_dict = USER_DICT.lock();
                    user_dict.add_phrase(&clean_pinyin, &raw_candidate_text);

                    // 2. Học cụm từ liên tiếp (chain): nếu trước đó cũng vừa commit tiếng Trung, ghép lại và lưu
                    if !state.cn_prev_committed_text.is_empty() {
                        let combined_text = format!("{}{}", state.cn_prev_committed_text, raw_candidate_text);
                        let combined_pinyin = format!("{}{}", state.cn_prev_committed_pinyin, clean_pinyin);
                        user_dict.add_phrase(&combined_pinyin, &combined_text);
                    }
                    user_dict.save();
                }

                // Cập nhật prev commit
                state.cn_prev_committed_text = raw_candidate_text.clone();
                state.cn_prev_committed_pinyin = clean_pinyin;

                send_unicode_string(&commit_text);
            } else {
                // Commit raw pinyin (không phải chữ Hán) → xóa chain
                send_unicode_string(&state.cn_raw_buffer);
                state.cn_prev_committed_text.clear();
                state.cn_prev_committed_pinyin.clear();
            }
            state.cn_raw_buffer.clear();
            state.cn_candidates.clear();
            state.current_page = 0;
            hide_candidate_popup();
            return 1;
        }

        // Enter key: Output raw pinyin text (break chain)
        if vk_code == VK_RETURN && !state.cn_raw_buffer.is_empty() {
            send_unicode_string(&state.cn_raw_buffer);
            state.cn_raw_buffer.clear();
            state.cn_candidates.clear();
            state.current_page = 0;
            state.cn_prev_committed_text.clear();
            state.cn_prev_committed_pinyin.clear();
            hide_candidate_popup();
            return 1;
        }

        // Backspace: Delete last character of pinyin buffer
        if vk_code == VK_BACK && !state.cn_raw_buffer.is_empty() {
            state.cn_raw_buffer.pop();
            if state.cn_raw_buffer.is_empty() {
                state.cn_candidates.clear();
                state.current_page = 0;
                hide_candidate_popup();
            } else {
                state.current_page = 0;
                {
                    let user_dict = USER_DICT.lock();
                    state.cn_candidates = CHINESE_ENGINE.lookup_with_user_dict(&state.cn_raw_buffer, &user_dict);
                }
                show_candidate_popup(
                    &state.cn_raw_buffer,
                    &state.cn_candidates,
                    state.current_page,
                    state.page_size,
                    &state.variant,
                );
            }
            return 1;
        }

        // Escape: Cancel pinyin buffer (break chain)
        if vk_code == VK_ESCAPE && !state.cn_raw_buffer.is_empty() {
            state.cn_raw_buffer.clear();
            state.cn_candidates.clear();
            state.current_page = 0;
            state.cn_prev_committed_text.clear();
            state.cn_prev_committed_pinyin.clear();
            hide_candidate_popup();
            return 1;
        }

        // Next Candidate Page: '=' / '+' / ']' / '.' / PageDown / Down Arrow / Right Arrow
        if !state.cn_raw_buffer.is_empty()
            && (vk_code == 0xBB || vk_code == 0xDD || vk_code == 0xBE || vk_code == 0x22 || vk_code == 0x28 || vk_code == 0x27)
        {
            let max_pages = (state.cn_candidates.len() + state.page_size - 1) / state.page_size;
            if max_pages > 0 && state.current_page + 1 < max_pages {
                state.current_page += 1;
                show_candidate_popup(
                    &state.cn_raw_buffer,
                    &state.cn_candidates,
                    state.current_page,
                    state.page_size,
                    &state.variant,
                );
            }
            return 1;
        }

        // Previous Candidate Page: '-' / '_' / '[' / ',' / PageUp / Up Arrow / Left Arrow
        if !state.cn_raw_buffer.is_empty()
            && (vk_code == 0xBD || vk_code == 0xDB || vk_code == 0xBC || vk_code == 0x21 || vk_code == 0x26 || vk_code == 0x25)
        {
            if state.current_page > 0 {
                state.current_page -= 1;
                show_candidate_popup(
                    &state.cn_raw_buffer,
                    &state.cn_candidates,
                    state.current_page,
                    state.page_size,
                    &state.variant,
                );
            }
            return 1;
        }

        // Other keys when pinyin buffer is active -> Commit 1st candidate + passthrough
        // This breaks the commit chain (e.g. user typed a punctuation or non-Chinese key)
        if !state.cn_raw_buffer.is_empty() {
            let index = state.current_page * state.page_size;
            if index < state.cn_candidates.len() {
                let candidate = &state.cn_candidates[index];
                let commit_text = if state.variant == "Traditional" {
                    convert_s2t(&candidate.text)
                } else {
                    candidate.text.clone()
                };
                send_unicode_string(&commit_text);
            } else {
                send_unicode_string(&state.cn_raw_buffer);
            }
            state.cn_raw_buffer.clear();
            state.cn_candidates.clear();
            state.current_page = 0;
            // Other key (punctuation, symbol) → break commit chain
            state.cn_prev_committed_text.clear();
            state.cn_prev_committed_pinyin.clear();
            hide_candidate_popup();
        }
    }

    CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param)
}

pub struct WindowsHookManager {
    _dummy: u8,
}

impl WindowsHookManager {
    pub fn new() -> Self {
        Self { _dummy: 0 }
    }

    pub fn start(&mut self) {
        std::thread::spawn(|| unsafe {
            use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
                RegisterHotKey, UnregisterHotKey,
                MOD_CONTROL, MOD_SHIFT, MOD_ALT, MOD_NOREPEAT,
            };
            use windows_sys::Win32::UI::WindowsAndMessaging::WM_HOTKEY;

            const HOTKEY_CTRL_SHIFT: i32 = 1001;
            const HOTKEY_ALT_SHIFT:  i32 = 1002;

            // Register Ctrl+Shift system-wide hotkey (works even when Webview2 has focus)
            // MOD_NOREPEAT prevents repeated fire while holding the keys
            RegisterHotKey(
                std::ptr::null_mut(),
                HOTKEY_CTRL_SHIFT,
                (MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT) as u32,
                0, // vkey = 0 means just the modifier combination
            );

            // Register Alt+Shift system-wide hotkey
            RegisterHotKey(
                std::ptr::null_mut(),
                HOTKEY_ALT_SHIFT,
                (MOD_ALT | MOD_SHIFT | MOD_NOREPEAT) as u32,
                0,
            );

            let kb_hook = SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(keyboard_proc),
                0 as isize as _,
                0,
            );

            let mouse_hook = SetWindowsHookExW(
                WH_MOUSE_LL,
                Some(mouse_proc),
                0 as isize as _,
                0,
            );

            if kb_hook == std::ptr::null_mut() {
                eprintln!("[VCKeys Hook] Failed to install low-level keyboard hook.");
                if mouse_hook != std::ptr::null_mut() {
                    UnhookWindowsHookEx(mouse_hook);
                }
                UnregisterHotKey(std::ptr::null_mut(), HOTKEY_CTRL_SHIFT);
                UnregisterHotKey(std::ptr::null_mut(), HOTKEY_ALT_SHIFT);
                return;
            }

            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                if msg.message == WM_HOTKEY {
                    match msg.wParam as i32 {
                        HOTKEY_CTRL_SHIFT => {
                            let mut state = GLOBAL_STATE.lock();
                            // Skip if cancelled (e.g. Ctrl+Shift+F was just pressed)
                            if state.ctrl_shift_cancelled {
                                state.ctrl_shift_cancelled = false;
                                continue;
                            }
                            let next_mode = if state.mode == "Vietnamese" {
                                "Chinese"
                            } else {
                                "Vietnamese"
                            };
                            state.mode = next_mode.to_string();
                            state.reset_vietnamese_buffers();
                            state.cn_raw_buffer.clear();
                            state.cn_candidates.clear();
                            let mode_to_announce = next_mode.to_string();
                            drop(state);
                            hide_candidate_popup();
                            crate::update_tray_icon_for_mode(&mode_to_announce);
                        }
                        HOTKEY_ALT_SHIFT => {
                            let mut state = GLOBAL_STATE.lock();
                            if state.alt_shift_cancelled {
                                state.alt_shift_cancelled = false;
                                continue;
                            }
                            state.mode = "English".to_string();
                            state.reset_vietnamese_buffers();
                            state.cn_raw_buffer.clear();
                            state.cn_candidates.clear();
                            drop(state);
                            hide_candidate_popup();
                            crate::update_tray_icon_for_mode("English");
                        }
                        _ => {}
                    }
                }
            }

            UnhookWindowsHookEx(kb_hook);
            if mouse_hook != std::ptr::null_mut() {
                UnhookWindowsHookEx(mouse_hook);
            }
            UnregisterHotKey(std::ptr::null_mut(), HOTKEY_CTRL_SHIFT);
            UnregisterHotKey(std::ptr::null_mut(), HOTKEY_ALT_SHIFT);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simulate_type(state: &mut HookState, ch: char) {
        state.vn_raw_word.push(ch);
        state.vn_displayed_str = VietnameseTelexEngine::transform(&state.vn_raw_word);
    }

    fn simulate_space(state: &mut HookState) {
        if !state.vn_raw_word.is_empty() {
            let entry = WordEntry {
                raw: state.vn_raw_word.clone(),
                displayed: state.vn_displayed_str.clone(),
                spaces_after: 1,
            };
            state.vn_word_history.push(entry);
            if state.vn_word_history.len() > 30 {
                state.vn_word_history.remove(0);
            }
            state.vn_raw_word.clear();
            state.vn_displayed_str.clear();
        } else if let Some(last) = state.vn_word_history.last_mut() {
            last.spaces_after += 1;
        }
    }

    fn simulate_backspace(state: &mut HookState) {
        if !state.vn_raw_word.is_empty() {
            let curr_len = state.vn_displayed_str.chars().count();
            let target_len = curr_len.saturating_sub(1);
            while !state.vn_raw_word.is_empty()
                && VietnameseTelexEngine::transform(&state.vn_raw_word).chars().count() > target_len
            {
                state.vn_raw_word.pop();
            }
            state.vn_displayed_str = VietnameseTelexEngine::transform(&state.vn_raw_word);
        } else if let Some(last) = state.vn_word_history.last_mut() {
            if last.spaces_after > 1 {
                last.spaces_after -= 1;
            } else {
                let entry = state.vn_word_history.pop().unwrap();
                state.vn_raw_word = entry.raw;
                state.vn_displayed_str = entry.displayed;
            }
        }
    }

    fn simulate_delete_word(state: &mut HookState) {
        let count = state.vn_displayed_str.chars().count();
        for _ in 0..count {
            simulate_backspace(state);
        }
    }

    #[test]
    fn test_cross_word_backspace_and_tone_change() {
        let mut state = HookState::default();

        // 1. Type "conf" -> "còn"
        for c in "conf".chars() {
            simulate_type(&mut state, c);
        }
        assert_eq!(state.vn_displayed_str, "còn");
        assert_eq!(state.vn_raw_word, "conf");

        // 2. Type space
        simulate_space(&mut state);
        assert_eq!(state.vn_raw_word, "");
        assert_eq!(state.vn_displayed_str, "");
        assert_eq!(state.vn_word_history.len(), 1);

        // 3. Type "dsad"
        for c in "dsad".chars() {
            simulate_type(&mut state, c);
        }
        assert_eq!(state.vn_displayed_str, "dsad");

        // 4. Backspace all characters of "dsad"
        simulate_delete_word(&mut state);
        assert_eq!(state.vn_raw_word, "");
        assert_eq!(state.vn_displayed_str, "");
        assert_eq!(state.vn_word_history.len(), 1);

        // 5. Backspace 1 more time to delete the space
        simulate_backspace(&mut state);
        assert_eq!(state.vn_word_history.len(), 0);
        assert_eq!(state.vn_raw_word, "conf");
        assert_eq!(state.vn_displayed_str, "còn");

        // 6. Type 's' -> should change "còn" to "cón"!
        simulate_type(&mut state, 's');
        assert_eq!(state.vn_raw_word, "confs");
        assert_eq!(state.vn_displayed_str, "cón");
    }

    #[test]
    fn test_all_tones_after_cross_word_restore() {
        let tone_cases = [
            ('s', "cón"),
            ('f', "conf"),
            ('r', "cỏn"),
            ('x', "cõn"),
            ('j', "cọn"),
        ];

        for (tone_key, expected_word) in tone_cases {
            let mut state = HookState::default();

            for c in "conf".chars() {
                simulate_type(&mut state, c);
            }
            simulate_space(&mut state);
            for c in "dsad".chars() {
                simulate_type(&mut state, c);
            }
            simulate_delete_word(&mut state);
            simulate_backspace(&mut state); // delete space

            assert_eq!(state.vn_raw_word, "conf");
            assert_eq!(state.vn_displayed_str, "còn");

            simulate_type(&mut state, tone_key);
            assert_eq!(state.vn_displayed_str, expected_word, "Failed for tone key '{}'", tone_key);
        }
    }

    #[test]
    fn test_multi_space_and_multi_word_history() {
        let mut state = HookState::default();

        // Type "hoc" + 2 spaces
        for c in "hoc".chars() { simulate_type(&mut state, c); }
        simulate_space(&mut state);
        simulate_space(&mut state);

        // Type "tap" + 1 space
        for c in "tap".chars() { simulate_type(&mut state, c); }
        simulate_space(&mut state);

        // Type "sai"
        for c in "sai".chars() { simulate_type(&mut state, c); }

        // Backspace "sai"
        simulate_delete_word(&mut state);
        // Backspace space between tap and sai
        simulate_backspace(&mut state);
        assert_eq!(state.vn_displayed_str, "tap");

        // Type 's' on tap -> "táp"
        simulate_type(&mut state, 's');
        assert_eq!(state.vn_displayed_str, "táp");

        // Now backspace "táp"
        simulate_delete_word(&mut state);
        assert_eq!(state.vn_raw_word, "");

        // Backspace 1st space after "hoc"
        simulate_backspace(&mut state);
        assert_eq!(state.vn_raw_word, ""); // still has 1 space left in history

        // Backspace 2nd space after "hoc" -> restores "hoc"!
        simulate_backspace(&mut state);
        assert_eq!(state.vn_displayed_str, "hoc");

        // Type 'j' on hoc -> "học"!
        simulate_type(&mut state, 'j');
        assert_eq!(state.vn_displayed_str, "học");
    }

    #[test]
    fn test_mouse_click_resets_vietnamese_buffers_and_modifiers() {
        let mut state = HookState::default();

        // Type a word and set modifiers
        for c in "xin".chars() {
            simulate_type(&mut state, c);
        }
        simulate_space(&mut state);
        for c in "chao".chars() {
            simulate_type(&mut state, c);
        }
        state.ctrl_down = true;
        state.alt_down = true;
        state.win_down = true;

        assert!(!state.vn_raw_word.is_empty());
        assert!(!state.vn_word_history.is_empty());

        // Simulate mouse click event
        state.reset_vietnamese_buffers();
        state.reset_modifiers();

        assert_eq!(state.vn_raw_word, "");
        assert_eq!(state.vn_displayed_str, "");
        assert_eq!(state.vn_word_history.len(), 0);
        assert!(!state.ctrl_down);
        assert!(!state.alt_down);
        assert!(!state.win_down);

        // Subsequent typing starts completely fresh with full diacritic support
        for c in "tieengs".chars() {
            simulate_type(&mut state, c);
        }
        assert_eq!(state.vn_displayed_str, "tiếng");
    }

    #[test]
    fn test_self_healing_buffer_boundary() {
        let mut state = HookState::default();

        // Simulate typing a long string (16 chars) without space (e.g. from pasting or missed reset)
        for c in "abcdefghijklmnop".chars() {
            if state.vn_raw_word.chars().count() >= 15 {
                state.reset_vietnamese_buffers();
            }
            simulate_type(&mut state, c);
        }

        // Buffer self-healed and only has the latest character 'p'
        assert_eq!(state.vn_raw_word, "p");

        // Now type a Vietnamese syllable: "hojc" -> "học"
        for c in "hojc".chars() {
            simulate_type(&mut state, c);
        }
        assert_eq!(state.vn_displayed_str, "phọc");
    }
}

