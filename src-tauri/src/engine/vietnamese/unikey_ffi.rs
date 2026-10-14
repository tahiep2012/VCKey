use std::sync::Mutex;
use std::sync::atomic::{AtomicI32, Ordering};
use lazy_static::lazy_static;

#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InputMethod {
    Telex = 0,
    Vni = 1,
    Viqr = 2,
    MsVi = 3,
    UsrIM = 4,
    SimpleTelex = 5,
    SimpleTelex2 = 6,
}

impl InputMethod {
    pub fn from_str(s: &str) -> Self {
        match s {
            "VNI" | "Vni" | "vni" => InputMethod::Vni,
            "Simple Telex" | "simple_telex" => InputMethod::SimpleTelex,
            "VIQR" | "Viqr" | "viqr" => InputMethod::Viqr,
            "Microsoft Vietnamese" | "msvi" => InputMethod::MsVi,
            _ => InputMethod::Telex,
        }
    }
}

pub static CURRENT_IM: AtomicI32 = AtomicI32::new(0); // 0 = Telex

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UnikeyOptions {
    pub free_marking: i32,
    pub modern_style: i32,
    pub macro_enabled: i32,
    pub use_unicode_clipboard: i32,
    pub always_macro: i32,
    pub strict_spell_check: i32,
    pub use_ime: i32,
    pub spell_check_enabled: i32,
    pub auto_non_vn_restore: i32,
}

extern "C" {
    static mut UnikeyBuf: [u8; 1024];
    static mut UnikeyBackspaces: i32;
    static mut UnikeyBufChars: i32;

    fn UnikeySetup();
    fn UnikeyCleanup();
    fn UnikeyResetBuf();
    fn UnikeyFilter(ch: u32);
    fn UnikeySetCapsState(shift_pressed: i32, caps_lock_on: i32);
    fn UnikeyBackspacePress();
    fn UnikeyRestoreKeyStrokes();
    fn UnikeySetInputMethod(im: i32);
    fn UnikeySetOptions(opt: *const UnikeyOptions);
    fn CreateDefaultUnikeyOptions(opt: *mut UnikeyOptions);
}

lazy_static! {
    static ref UNIKEY_MUTEX: Mutex<()> = {
        unsafe {
            UnikeySetup();
            let mut opts = UnikeyOptions {
                free_marking: 1,
                modern_style: 0,
                macro_enabled: 0,
                use_unicode_clipboard: 0,
                always_macro: 0,
                strict_spell_check: 0,
                use_ime: 0,
                spell_check_enabled: 1,
                auto_non_vn_restore: 0,
            };
            CreateDefaultUnikeyOptions(&mut opts);
            opts.free_marking = 1;
            opts.spell_check_enabled = 1;
            opts.auto_non_vn_restore = 1;
            UnikeySetOptions(&opts);
            UnikeySetInputMethod(0); // 0 = UkTelex
        }
        Mutex::new(())
    };
}

pub struct UnikeyEngine;

impl UnikeyEngine {
    pub fn set_input_method(method: InputMethod) {
        let _guard = UNIKEY_MUTEX.lock().unwrap();
        unsafe {
            UnikeySetInputMethod(method as i32);
            CURRENT_IM.store(method as i32, Ordering::Release);
        }
    }

    /// Apply smart-typing options from AppSettings to the UniKey C++ engine.
    /// NOTE: In this UniKey build, modernStyle=0 gives new-standard (hóa, thủy)
    /// and modernStyle=1 gives old-standard (hoá, thuỷ) — opposite of the field name.
    /// We invert the flag accordingly.
    pub fn set_options(free_mark: bool, spell_check: bool, auto_english_restore: bool, modern_style: bool) {
        let _guard = UNIKEY_MUTEX.lock().unwrap();
        unsafe {
            let mut opts = UnikeyOptions {
                free_marking: if free_mark { 1 } else { 0 },
                // INVERTED: modernStyle=0 → new standard (hóa), modernStyle=1 → old standard (hoá)
                modern_style: if modern_style { 0 } else { 1 },
                macro_enabled: 0,
                use_unicode_clipboard: 0,
                always_macro: 0,
                strict_spell_check: if spell_check { 0 } else { 1 },
                use_ime: 0,
                spell_check_enabled: if spell_check { 1 } else { 0 },
                auto_non_vn_restore: if auto_english_restore { 1 } else { 0 },
            };
            CreateDefaultUnikeyOptions(&mut opts);
            // Override after CreateDefault to ensure our values stick
            opts.free_marking = if free_mark { 1 } else { 0 };
            opts.modern_style = if modern_style { 0 } else { 1 }; // INVERTED
            opts.spell_check_enabled = if spell_check { 1 } else { 0 };
            opts.auto_non_vn_restore = if auto_english_restore { 1 } else { 0 };
            UnikeySetOptions(&opts);
        }
    }

    pub fn get_input_method() -> InputMethod {
        match CURRENT_IM.load(Ordering::Acquire) {
            1 => InputMethod::Vni,
            2 => InputMethod::Viqr,
            3 => InputMethod::MsVi,
            5 => InputMethod::SimpleTelex,
            6 => InputMethod::SimpleTelex2,
            _ => InputMethod::Telex,
        }
    }

    pub fn transform(raw: &str) -> String {
        Self::transform_with_method(raw, Self::get_input_method())
    }

    pub fn transform_with_method(raw: &str, method: InputMethod) -> String {
        if raw.is_empty() {
            return String::new();
        }
        let _guard = UNIKEY_MUTEX.lock().unwrap();
        unsafe {
            UnikeySetInputMethod(method as i32);
            UnikeyResetBuf();
            let mut out = String::new();
            for ch in raw.chars() {
                let is_upper = ch.is_uppercase();
                UnikeySetCapsState(if is_upper { 1 } else { 0 }, 0);
                UnikeyFilter(ch as u32);
                let backspaces = UnikeyBackspaces.max(0) as usize;
                for _ in 0..backspaces {
                    out.pop();
                }
                if UnikeyBufChars > 0 {
                    if let Ok(s) = std::str::from_utf8(&UnikeyBuf[0..UnikeyBufChars as usize]) {
                        out.push_str(s);
                    }
                } else {
                    out.push(ch);
                }
            }
            let cur = CURRENT_IM.load(Ordering::Acquire);
            if cur != method as i32 {
                UnikeySetInputMethod(cur);
            }
            out
        }
    }

    pub fn reset() {
        let _guard = UNIKEY_MUTEX.lock().unwrap();
        unsafe {
            UnikeyResetBuf();
        }
    }
}
