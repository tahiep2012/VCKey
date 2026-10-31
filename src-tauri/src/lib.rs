pub mod engine;

use std::sync::Arc;
use parking_lot::Mutex;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};
use crate::engine::config::{AppSettings, ConfigStore};
use crate::engine::hook::{GLOBAL_STATE, WindowsHookManager};

lazy_static::lazy_static! {
    pub static ref GLOBAL_APP_HANDLE: Arc<Mutex<Option<tauri::AppHandle>>> = Arc::new(Mutex::new(None));
}

use std::sync::atomic::{AtomicU8, Ordering};

pub const MODE_VIETNAMESE: u8 = 0;
pub const MODE_CHINESE: u8 = 1;
pub const MODE_ENGLISH: u8 = 2;

pub static CURRENT_MODE_ATOMIC: AtomicU8 = AtomicU8::new(MODE_VIETNAMESE);

pub fn mode_to_str(m: u8) -> &'static str {
    match m {
        MODE_CHINESE => "Chinese",
        MODE_ENGLISH => "English",
        _ => "Vietnamese",
    }
}

pub fn str_to_mode(s: &str) -> u8 {
    match s {
        "Chinese" => MODE_CHINESE,
        "English" => MODE_ENGLISH,
        _ => MODE_VIETNAMESE,
    }
}

pub fn update_tray_icon_for_mode(mode: &str) {
    let mode_owned = mode.to_string();
    CURRENT_MODE_ATOMIC.store(str_to_mode(mode), Ordering::Release);

    std::thread::spawn(move || {
        if let Some(app) = GLOBAL_APP_HANDLE.lock().as_ref() {
            let _ = app.emit("mode-changed", &mode_owned);
            if let Some(tray) = app.tray_by_id("main_tray") {
                let icon_bytes: &[u8] = match mode_owned.as_str() {
                    "Chinese" => include_bytes!("../icons/tray_cn.png"),
                    "English" => include_bytes!("../icons/tray_en.png"),
                    _ => include_bytes!("../icons/tray_vn.png"),
                };
                if let Ok(img) = tauri::image::Image::from_bytes(icon_bytes) {
                    let _ = tray.set_icon(Some(img));
                    let tooltip = match mode_owned.as_str() {
                        "Chinese" => "VCKeys 3.0.2 - Tiếng Trung (Pinyin)",
                        "English" => "VCKeys 3.0.2 - Tiếng Anh (English)",
                        _ => "VCKeys 3.0.2 - Tiếng Việt (Telex)",
                    };
                    let _ = tray.set_tooltip(Some(tooltip));
                }
            }
        }

        // Show native OSD Toast notification in bottom-right corner
        crate::engine::osd::show_osd_notification(&mode_owned);
    });
}

fn get_config_dir() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

#[tauri::command]
fn get_settings() -> AppSettings {
    let store = ConfigStore::new(get_config_dir());
    store.load_settings()
}

#[tauri::command]
fn save_settings(settings: AppSettings) -> Result<(), String> {
    let store = ConfigStore::new(get_config_dir());
    store.save_settings(&settings)?;
    {
        let mut state = GLOBAL_STATE.lock();
        state.mode = settings.mode.clone();
        state.page_size = settings.cand_page_size;
        state.font_size = settings.cand_font_size;
        state.variant = if settings.traditional_chinese {
            "Traditional".to_string()
        } else {
            "Simplified".to_string()
        };
        state.chinese_punctuation = settings.chinese_punctuation;
        state.vn_auto_english_restore = settings.vn_auto_english_restore;
    }
    {
        let im = crate::engine::vietnamese::unikey_ffi::InputMethod::from_str(&settings.input_type);
        crate::engine::vietnamese::unikey_ffi::UnikeyEngine::set_input_method(im);
    }
    // Apply smart-typing options to UniKey C++ engine
    crate::engine::vietnamese::unikey_ffi::UnikeyEngine::set_options(
        settings.vn_free_mark,
        settings.vn_spelling_check,
        settings.vn_auto_english_restore,
        settings.vn_new_accent_style,
    );
    // Update English-restore runtime flag
    crate::engine::vietnamese::telex::VN_AUTO_ENGLISH_RESTORE
        .store(settings.vn_auto_english_restore, std::sync::atomic::Ordering::Relaxed);
    {
        let mut cand_state = crate::engine::chinese::candidate_window::CANDIDATE_UI_STATE.lock();
        cand_state.page_size = settings.cand_page_size;
        cand_state.font_size = settings.cand_font_size;
        cand_state.theme = settings.theme.clone();
        cand_state.show_pinyin_tone = settings.chinese_show_pinyin_tone;
    }
    // Apply Windows autostart registry entry
    apply_autostart(settings.run_on_startup)?;
    update_tray_icon_for_mode(&settings.mode);
    Ok(())
}

/// Write or delete the autostart registry entry under
/// HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run
fn apply_autostart(enable: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Registry::{
            RegOpenKeyExW, RegSetValueExW, RegDeleteValueW, RegCloseKey,
            HKEY_CURRENT_USER, KEY_SET_VALUE, REG_SZ,
        };
        let subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run\0"
            .encode_utf16().collect();
        let value_name: Vec<u16> = "VCKeys\0".encode_utf16().collect();
        let mut hkey: windows_sys::Win32::System::Registry::HKEY = std::ptr::null_mut();
        let res = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                subkey.as_ptr(),
                0,
                KEY_SET_VALUE,
                &mut hkey,
            )
        };
        if res != 0 {
            return Err(format!("Không thể mở registry key: lỗi {}", res));
        }
        if enable {
            // Get current exe path
            let exe_path = std::env::current_exe()
                .map_err(|e| e.to_string())?;
            let exe_str = format!("\"{}\"", exe_path.to_string_lossy());
            let exe_wide: Vec<u16> = exe_str.encode_utf16().chain(std::iter::once(0)).collect();
            let res2 = unsafe {
                RegSetValueExW(
                    hkey,
                    value_name.as_ptr(),
                    0,
                    REG_SZ,
                    exe_wide.as_ptr() as *const u8,
                    (exe_wide.len() * 2) as u32,
                )
            };
            unsafe { RegCloseKey(hkey) };
            if res2 != 0 {
                return Err(format!("Không thể ghi registry: lỗi {}", res2));
            }
        } else {
            let res2 = unsafe { RegDeleteValueW(hkey, value_name.as_ptr()) };
            unsafe { RegCloseKey(hkey) };
            // Ignore ERROR_FILE_NOT_FOUND (2) — key simply wasn't there
            if res2 != 0 && res2 != 2 {
                return Err(format!("Không thể xóa registry: lỗi {}", res2));
            }
        }
    }
    Ok(())
}


#[tauri::command]
fn set_mode(mode: String) -> Result<(), String> {
    {
        let mut state = GLOBAL_STATE.lock();
        state.mode = mode.clone();
        state.reset_vietnamese_buffers();
        state.cn_raw_buffer.clear();
        state.cn_candidates.clear();
        crate::engine::chinese::hide_candidate_popup();
    }
    update_tray_icon_for_mode(&mode);
    Ok(())
}

pub fn toggle_chinese_variant() -> String {
    let store = ConfigStore::new(get_config_dir());
    let mut settings = store.load_settings();
    settings.traditional_chinese = !settings.traditional_chinese;
    let _ = store.save_settings(&settings);

    let next_variant = if settings.traditional_chinese {
        "Traditional"
    } else {
        "Simplified"
    };

    {
        let mut state = GLOBAL_STATE.lock();
        state.variant = next_variant.to_string();
    }

    if let Some(app) = GLOBAL_APP_HANDLE.lock().as_ref() {
        let _ = app.emit("settings-updated", &settings);
    }

    let (tag, msg, color) = if next_variant == "Traditional" {
        ("CN", "VCKeys: PHỒN THỂ (繁體)", 0x00EB6325u32)
    } else {
        ("CN", "VCKeys: GIẢN THỂ (简体)", 0x00EB6325u32)
    };
    crate::engine::osd::show_osd_custom(tag, msg, color);

    next_variant.to_string()
}

#[tauri::command]
fn toggle_variant() -> Result<String, String> {
    Ok(toggle_chinese_variant())
}

#[tauri::command]
fn get_current_mode() -> String {
    mode_to_str(CURRENT_MODE_ATOMIC.load(Ordering::Acquire)).to_string()
}

#[tauri::command]
fn set_theme(theme: String) -> Result<(), String> {
    let store = ConfigStore::new(get_config_dir());
    let mut settings = store.load_settings();
    settings.theme = theme.clone();
    let _ = store.save_settings(&settings);
    {
        let mut cand_state = crate::engine::chinese::candidate_window::CANDIDATE_UI_STATE.lock();
        cand_state.theme = theme;
    }
    let hwnd = crate::engine::chinese::candidate_window::CAND_HWND.load(Ordering::SeqCst) as windows_sys::Win32::Foundation::HWND;
    if hwnd != std::ptr::null_mut() {
        unsafe {
            windows_sys::Win32::Graphics::Gdi::InvalidateRect(hwnd, std::ptr::null(), 1);
        }
    }
    Ok(())
}

#[tauri::command]
fn exit_app(app: tauri::AppHandle) {
    app.exit(0);
}

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

const SINGLE_INSTANCE_PORT: u16 = 28765;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Single-instance enforcement via local socket communication
    match TcpListener::bind(("127.0.0.1", SINGLE_INSTANCE_PORT)) {
        Ok(listener) => {
            // First instance: listen in background to show main window when secondary instances launch
            std::thread::spawn(move || {
                for stream in listener.incoming() {
                    if let Ok(mut s) = stream {
                        let mut buf = [0u8; 16];
                        let _ = s.read(&mut buf);
                        if let Some(app) = GLOBAL_APP_HANDLE.lock().as_ref() {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                    }
                }
            });
        }
        Err(_) => {
            if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", SINGLE_INSTANCE_PORT)) {
                let _ = stream.write_all(b"SHOW");
            }
            std::process::exit(0);
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            set_mode,
            set_theme,
            toggle_variant,
            get_current_mode,
            exit_app
        ])
        .setup(|app| {
            *GLOBAL_APP_HANDLE.lock() = Some(app.handle().clone());

            // Enforce compact window size & disable maximize button
            if let Some(main_win) = app.get_webview_window("main") {
                let _ = main_win.set_maximizable(false);
                let _ = main_win.set_resizable(false);
            }

            // Load saved settings and sync to global states
            let store = ConfigStore::new(get_config_dir());
            let saved_settings = store.load_settings();
            {
                let mut state = GLOBAL_STATE.lock();
                state.mode = saved_settings.mode.clone();
                state.page_size = saved_settings.cand_page_size;
                state.font_size = saved_settings.cand_font_size;
                state.variant = if saved_settings.traditional_chinese {
                    "Traditional".to_string()
                } else {
                    "Simplified".to_string()
                };
                state.chinese_punctuation = saved_settings.chinese_punctuation;
                state.vn_auto_english_restore = saved_settings.vn_auto_english_restore;
            }
            {
                let im = crate::engine::vietnamese::unikey_ffi::InputMethod::from_str(&saved_settings.input_type);
                crate::engine::vietnamese::unikey_ffi::UnikeyEngine::set_input_method(im);
            }
            // Apply smart-typing options from saved settings
            crate::engine::vietnamese::unikey_ffi::UnikeyEngine::set_options(
                saved_settings.vn_free_mark,
                saved_settings.vn_spelling_check,
                saved_settings.vn_auto_english_restore,
                saved_settings.vn_new_accent_style,
            );
            crate::engine::vietnamese::telex::VN_AUTO_ENGLISH_RESTORE
                .store(saved_settings.vn_auto_english_restore, std::sync::atomic::Ordering::Relaxed);
            {
                let mut cand_state = crate::engine::chinese::candidate_window::CANDIDATE_UI_STATE.lock();
                cand_state.page_size = saved_settings.cand_page_size;
                cand_state.font_size = saved_settings.cand_font_size;
                cand_state.theme = saved_settings.theme.clone();
                cand_state.show_pinyin_tone = saved_settings.chinese_show_pinyin_tone;
            }

            // Start the Win32 low-level keyboard hook
            let mut hook_mgr = WindowsHookManager::new();
            hook_mgr.start();

            // Start the native Win32 Candidate Popup window thread
            crate::engine::chinese::start_candidate_window_thread();

            // Start the native Win32 OSD Toast notification thread
            crate::engine::osd::start_osd_window_thread();

            // Pre-warm the Chinese pinyin dictionary in the background so the first
            // keystroke in Chinese mode is instant (dict.txt is ~12MB / 457K lines;
            // without warmup the lazy_static blocks on first keypress for 1-3 seconds).
            std::thread::Builder::new()
                .name("chinese-warmup".to_string())
                .spawn(|| {
                    // Accessing CHINESE_ENGINE forces its lazy_static to initialise,
                    // which parses dict.txt and builds the HashMap / jianpin index.
                    let _ = crate::engine::hook::windows_hook::CHINESE_ENGINE.lookup("ni");
                })
                .ok();


            // Build System Tray Menu
            let show_item = MenuItem::with_id(app, "show", "Bảng điều khiển VCKeys", true, None::<&str>)?;
            let sep1 = PredefinedMenuItem::separator(app)?;
            let vn_item = MenuItem::with_id(app, "mode_vn", "🇻🇳 Tiếng Việt (Telex)", true, None::<&str>)?;
            let cn_item = MenuItem::with_id(app, "mode_cn", "🇨🇳 Tiếng Trung (Pinyin)", true, None::<&str>)?;
            let en_item = MenuItem::with_id(app, "mode_en", "🇬🇧 Tiếng Anh (English)", true, None::<&str>)?;
            let sep2 = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "Thoát VCKeys", true, None::<&str>)?;

            let tray_menu = Menu::with_items(app, &[
                &show_item,
                &sep1,
                &vn_item,
                &cn_item,
                &en_item,
                &sep2,
                &quit_item,
            ])?;

            let initial_icon_bytes = include_bytes!("../icons/tray_vn.png");
            let initial_icon = tauri::image::Image::from_bytes(initial_icon_bytes)
                .unwrap_or_else(|_| app.default_window_icon().unwrap().clone());

            let _tray = TrayIconBuilder::with_id("main_tray")
                .icon(initial_icon)
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .tooltip("VCKeys 3.0.2 - Tiếng Việt (Telex)")
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                        "mode_vn" => {
                            let mut state = GLOBAL_STATE.lock();
                            state.mode = "Vietnamese".to_string();
                            state.reset_vietnamese_buffers();
                            drop(state);
                            update_tray_icon_for_mode("Vietnamese");
                        }
                        "mode_cn" => {
                            let mut state = GLOBAL_STATE.lock();
                            state.mode = "Chinese".to_string();
                            state.cn_raw_buffer.clear();
                            drop(state);
                            update_tray_icon_for_mode("Chinese");
                        }
                        "mode_en" => {
                            let mut state = GLOBAL_STATE.lock();
                            state.mode = "English".to_string();
                            drop(state);
                            update_tray_icon_for_mode("English");
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                                trim_memory();
                            } else {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // Continuous background memory governor:
            // When the UI is hidden in tray, periodically flushes working set every 30 seconds
            // to permanently prevent WebView2 Chromium background tasks from growing RAM back up.
            std::thread::Builder::new()
                .name("memory-governor".to_string())
                .spawn(|| {
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    trim_memory();
                    loop {
                        std::thread::sleep(std::time::Duration::from_secs(30));
                        let is_hidden = if let Some(app) = GLOBAL_APP_HANDLE.lock().as_ref() {
                            if let Some(window) = app.get_webview_window("main") {
                                !window.is_visible().unwrap_or(true)
                            } else {
                                true
                            }
                        } else {
                            false
                        };

                        if is_hidden {
                            trim_memory();
                        }
                    }
                })
                .ok();

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // Prevent app from exiting on close button 'X', minimize to system tray instead
                api.prevent_close();
                let _ = window.hide();
                trim_memory();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running vckeys tauri application");
}

pub fn trim_memory() {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
        use windows_sys::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        };
        use windows_sys::Win32::System::Threading::{
            GetCurrentProcess, GetCurrentProcessId, OpenProcess, SetProcessWorkingSetSize,
            PROCESS_QUERY_INFORMATION, PROCESS_SET_QUOTA,
        };

        // 1. Trim the main VCKeys process
        SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX);

        // 2. Take process snapshot to discover all child & grandchild processes (WebView2, GPU, renderers)
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot != INVALID_HANDLE_VALUE {
            let my_pid = GetCurrentProcessId();
            let mut pids_to_check = vec![my_pid];
            let mut descendant_pids = Vec::new();

            let mut all_processes: Vec<(u32, u32)> = Vec::with_capacity(256);
            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                cntUsage: 0,
                th32ProcessID: 0,
                th32DefaultHeapID: 0,
                th32ModuleID: 0,
                cntThreads: 0,
                th32ParentProcessID: 0,
                pcPriClassBase: 0,
                dwFlags: 0,
                szExeFile: [0; 260],
            };

            if Process32FirstW(snapshot, &mut entry) != 0 {
                loop {
                    all_processes.push((entry.th32ProcessID, entry.th32ParentProcessID));
                    entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
                    if Process32NextW(snapshot, &mut entry) == 0 {
                        break;
                    }
                }
            }
            CloseHandle(snapshot);

            // Recursively collect all descendant PIDs (WebView2 browser, renderer, GPU, utility processes)
            while let Some(parent) = pids_to_check.pop() {
                for &(pid, p_parent) in &all_processes {
                    if p_parent == parent && !descendant_pids.contains(&pid) {
                        descendant_pids.push(pid);
                        pids_to_check.push(pid);
                    }
                }
            }

            // 3. Trim each child and descendant process working set
            for pid in descendant_pids {
                let h_proc = OpenProcess(PROCESS_SET_QUOTA | PROCESS_QUERY_INFORMATION, 0, pid);
                if !h_proc.is_null() {
                    SetProcessWorkingSetSize(h_proc, usize::MAX, usize::MAX);
                    CloseHandle(h_proc);
                }
            }
        }
    }
}

