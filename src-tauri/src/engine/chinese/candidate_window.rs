use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::Arc;
use parking_lot::Mutex;
use windows_sys::Win32::Foundation::{CloseHandle, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, ClientToScreen, CreateFontW, CreatePen, CreateSolidBrush, DeleteObject,
    DrawTextW, EndPaint, GetDC, GetTextExtentPoint32W, InvalidateRect, ReleaseDC, RoundRect, SelectObject, SetBkMode, SetTextColor,
    DT_CENTER, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, FW_BOLD, FW_NORMAL, PAINTSTRUCT,
    PS_SOLID, TRANSPARENT, ScreenToClient,
};
use windows_sys::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, CreateWindowExW, DefWindowProcW, DispatchMessageW, FindWindowW, GetClassNameW, GetCursorPos, GetForegroundWindow,
    GetGUIThreadInfo, GetMessageW, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId, LoadCursorW, RegisterClassExW,
    SetCursor, SetWindowPos, ShowWindow, SystemParametersInfoW, TranslateMessage, CS_HREDRAW, CS_VREDRAW, GUITHREADINFO,
    IDC_ARROW, IDC_HAND, SPI_GETWORKAREA, SWP_NOACTIVATE, SWP_SHOWWINDOW, SW_HIDE, SW_SHOWNOACTIVATE,
    WM_LBUTTONDOWN, WM_MOUSEMOVE, WM_PAINT, WM_SETCURSOR, WNDCLASSEXW,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};
use crate::engine::chinese::{Candidate, convert_s2t};
use crate::engine::hook::input_sender::send_unicode_string;

const WM_MOUSEACTIVATE: u32 = 0x0021;
const MA_NOACTIVATE: LRESULT = 3;

pub struct CandidateWindowState {
    pub visible: bool,
    pub x: i32,
    pub y: i32,
    pub preedit: String,
    pub candidates: Vec<Candidate>,
    pub current_page: usize,
    pub page_size: usize,
    pub font_size: usize,
    pub variant: String,
    pub theme: String,
    pub candidate_rects: Vec<(usize, RECT)>,
    pub prev_page_rect: RECT,
    pub next_page_rect: RECT,
}

impl Default for CandidateWindowState {
    fn default() -> Self {
        Self {
            visible: false,
            x: 100,
            y: 100,
            preedit: String::new(),
            candidates: Vec::new(),
            current_page: 0,
            page_size: 5,
            font_size: 17,
            variant: "Simplified".to_string(),
            theme: "system".to_string(),
            candidate_rects: Vec::new(),
            prev_page_rect: RECT { left: 0, top: 0, right: 0, bottom: 0 },
            next_page_rect: RECT { left: 0, top: 0, right: 0, bottom: 0 },
        }
    }
}

pub fn is_dark_mode_active(theme_setting: &str) -> bool {
    if theme_setting.eq_ignore_ascii_case("dark") {
        return true;
    }
    if theme_setting.eq_ignore_ascii_case("light") {
        return false;
    }
    // "system" -> check Windows Registry
    #[cfg(windows)]
    {
        extern "system" {
            fn RegOpenKeyExW(
                hKey: isize,
                lpSubKey: *const u16,
                ulOptions: u32,
                samDesired: u32,
                phkResult: *mut isize,
            ) -> i32;
            fn RegQueryValueExW(
                hKey: isize,
                lpValueName: *const u16,
                lpReserved: *mut u32,
                lpType: *mut u32,
                lpData: *mut u8,
                lpcbData: *mut u32,
            ) -> i32;
            fn RegCloseKey(hKey: isize) -> i32;
        }
        const HKEY_CURRENT_USER: isize = 0x80000001u32 as i32 as isize;
        const KEY_READ: u32 = 0x20019;

        let subkey = windows_sys::w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
        let mut hkey: isize = 0;
        unsafe {
            if RegOpenKeyExW(HKEY_CURRENT_USER, subkey, 0, KEY_READ, &mut hkey) == 0 {
                let value_name = windows_sys::w!("AppsUseLightTheme");
                let mut data: u32 = 1;
                let mut data_size = std::mem::size_of::<u32>() as u32;
                let mut val_type: u32 = 0;
                let res = RegQueryValueExW(
                    hkey,
                    value_name,
                    std::ptr::null_mut(),
                    &mut val_type,
                    &mut data as *mut u32 as *mut u8,
                    &mut data_size,
                );
                RegCloseKey(hkey);
                if res == 0 {
                    return data == 0; // 0 = Dark, 1 = Light
                }
            }
        }
    }
    false
}

lazy_static::lazy_static! {
    pub static ref CANDIDATE_UI_STATE: Arc<Mutex<CandidateWindowState>> = Arc::new(Mutex::new(CandidateWindowState::default()));
    pub static ref CAND_HWND: AtomicIsize = AtomicIsize::new(0);
}

pub unsafe fn detect_start_or_search_flyout(fg: HWND) -> Option<RECT> {
    if fg == std::ptr::null_mut() {
        return None;
    }

    let mut is_target = false;

    // 1. Check window class name
    let mut class_buf = [0u16; 256];
    let len = GetClassNameW(fg, class_buf.as_mut_ptr(), 256);
    if len > 0 {
        let class_name = String::from_utf16_lossy(&class_buf[..len as usize]);
        if class_name.contains("Windows.UI.Core.CoreWindow")
            || class_name.contains("Xaml_WindowedPopupClass")
            || class_name.contains("Search")
            || class_name.contains("Start")
            || class_name.contains("PopupHost")
        {
            is_target = true;
        }
    }

    // 2. Check window title
    if !is_target {
        let mut title_buf = [0u16; 256];
        let t_len = GetWindowTextW(fg, title_buf.as_mut_ptr(), 256);
        if t_len > 0 {
            let title = String::from_utf16_lossy(&title_buf[..t_len as usize]).to_lowercase();
            if title.contains("search") || title.contains("start") || title.contains("tìm kiếm") {
                is_target = true;
            }
        }
    }

    // 3. Check process name from PID
    if !is_target {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(fg, &mut pid);
        if pid != 0 {
            let h_proc = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if h_proc != std::ptr::null_mut() {
                let mut exe_buf = [0u16; 512];
                let mut size = 512u32;
                let success = QueryFullProcessImageNameW(h_proc, 0, exe_buf.as_mut_ptr(), &mut size);
                CloseHandle(h_proc);
                if success != 0 {
                    let exe_name = String::from_utf16_lossy(&exe_buf[..size as usize]).to_lowercase();
                    if exe_name.contains("searchhost")
                        || exe_name.contains("startmenuexperiencehost")
                        || exe_name.contains("shellexperiencehost")
                    {
                        is_target = true;
                    }
                }
            }
        }
    }

    if is_target {
        let mut rc: RECT = std::mem::zeroed();
        GetWindowRect(fg, &mut rc);
        if rc.right > rc.left && rc.bottom > rc.top {
            return Some(rc);
        }
    }

    None
}

pub fn get_caret_pos() -> (i32, i32) {
    unsafe {
        let fg = GetForegroundWindow();
        if fg != std::ptr::null_mut() {
            let tid = GetWindowThreadProcessId(fg, std::ptr::null_mut());
            let mut gti: GUITHREADINFO = std::mem::zeroed();
            gti.cbSize = std::mem::size_of::<GUITHREADINFO>() as u32;

            // Strategy 1: Win32 GUI Thread Info (Notepad, WordPad, Win32 Text Boxes)
            if GetGUIThreadInfo(tid, &mut gti) != 0 {
                let target_hwnd = if gti.hwndCaret != std::ptr::null_mut() {
                    gti.hwndCaret
                } else if gti.hwndFocus != std::ptr::null_mut() {
                    gti.hwndFocus
                } else {
                    fg
                };

                // Win32 Caret rect check: standard carets have height >= 2 or width >= 1
                if target_hwnd != std::ptr::null_mut()
                    && (gti.rcCaret.bottom > gti.rcCaret.top || gti.rcCaret.right > gti.rcCaret.left)
                {
                    let mut pt = POINT {
                        x: gti.rcCaret.left,
                        y: gti.rcCaret.bottom,
                    };
                    ClientToScreen(target_hwnd, &mut pt);
                    if pt.x > 0 || pt.y > 0 {
                        return (pt.x, pt.y + 4);
                    }
                }
            }

            // Strategy 2: Check for Windows 10/11 Start Menu, Taskbar Search Flyout, Cortana
            if let Some(rc_flyout) = detect_start_or_search_flyout(fg) {
                let mut pt_cursor: POINT = std::mem::zeroed();
                GetCursorPos(&mut pt_cursor);
                if pt_cursor.x >= rc_flyout.left && pt_cursor.x <= rc_flyout.right
                    && pt_cursor.y >= rc_flyout.top && pt_cursor.y <= rc_flyout.bottom {
                    return (pt_cursor.x, pt_cursor.y + 16);
                }
            }

            // Strategy 3: Foreground Window bounds & Mouse cursor proximity (VS Code, Chrome, Electron, Office 365)
            let mut rc_fg: RECT = std::mem::zeroed();
            GetWindowRect(fg, &mut rc_fg);
            let mut pt_cursor: POINT = std::mem::zeroed();
            GetCursorPos(&mut pt_cursor);

            // If mouse was clicked within the active application window
            if pt_cursor.x >= rc_fg.left - 40 && pt_cursor.x <= rc_fg.right + 40
                && pt_cursor.y >= rc_fg.top - 40 && pt_cursor.y <= rc_fg.bottom + 60 {
                return (pt_cursor.x, pt_cursor.y + 18);
            } else if rc_fg.right > rc_fg.left && rc_fg.bottom > rc_fg.top {
                // If cursor is outside, default to top-left area of the active window
                let default_x = (rc_fg.left + 40).max(10);
                let default_y = (rc_fg.top + 60).max(10);
                return (default_x, default_y);
            }
        }

        let mut pt: POINT = std::mem::zeroed();
        GetCursorPos(&mut pt);
        (pt.x, pt.y + 18)
    }
}

pub fn show_candidate_popup(
    preedit: &str,
    candidates: &[Candidate],
    page: usize,
    page_size: usize,
    variant: &str,
) {
    let (cx, cy) = get_caret_pos();
    let font_size = {
        let mut state = CANDIDATE_UI_STATE.lock();
        state.visible = true;
        state.x = cx;
        state.y = cy;
        state.preedit = preedit.to_string();
        state.candidates = candidates.to_vec();
        state.current_page = page;
        state.page_size = page_size;
        state.variant = variant.to_string();
        state.font_size
    };

    let hwnd = CAND_HWND.load(Ordering::SeqCst) as HWND;
    if hwnd != std::ptr::null_mut() {
        unsafe {
            let fs = (font_size as i32).max(12);
            let cand_fs = -fs;
            let start = page * page_size;
            let end = (start + page_size).min(candidates.len());
            let cand_item_gap = (fs * 4 / 10).max(6);

            let hdc = GetDC(hwnd);
            let preedit_fs = (fs * 14 / 17).max(12);
            let h_font_preedit = CreateFontW(
                -preedit_fs, 0, 0, 0, FW_BOLD as i32, 0, 0, 0, 1, 0, 0, 0, 0,
                windows_sys::w!("Segoe UI"),
            );
            let old_font = SelectObject(hdc, h_font_preedit);
            let preedit_utf16: Vec<u16> = preedit.encode_utf16().collect();
            let mut preedit_size: SIZE = std::mem::zeroed();
            GetTextExtentPoint32W(hdc, preedit_utf16.as_ptr(), preedit_utf16.len() as i32, &mut preedit_size);

            let h_font_cand = CreateFontW(
                cand_fs, 0, 0, 0, FW_NORMAL as i32, 0, 0, 0, 1, 0, 0, 0, 0,
                windows_sys::w!("Microsoft YaHei"),
            );
            SelectObject(hdc, h_font_cand);
            let mut total_cand_width = 0;
            let cand_count = end - start;
            for (i, c) in candidates[start..end].iter().enumerate() {
                let num = i + 1;
                let cand_text = if variant == "Traditional" {
                    convert_s2t(&c.text)
                } else {
                    c.text.clone()
                };
                let cand_str = format!("{}.{}", num, cand_text);
                let cand_utf16: Vec<u16> = cand_str.encode_utf16().collect();
                let mut size: SIZE = std::mem::zeroed();
                GetTextExtentPoint32W(hdc, cand_utf16.as_ptr(), cand_utf16.len() as i32, &mut size);
                total_cand_width += size.cx;
                if i < cand_count - 1 {
                    total_cand_width += cand_item_gap;
                }
            }

            SelectObject(hdc, old_font);
            DeleteObject(h_font_cand);
            DeleteObject(h_font_preedit);
            ReleaseDC(hwnd, hdc);

            let header_min_width = preedit_size.cx + 120; // preedit + 10px margin + 6px gap + 96px header controls + 8px right margin
            let cand_row_total = total_cand_width + 24; // 12px left + 12px right margin
            let width = cand_row_total.max(header_min_width).max(160);

            let header_h = (fs * 15 / 17).max(18);
            let cand_row_h = fs + 8;
            let height = header_h + cand_row_h + 12;

            let mut final_x = cx;
            let mut final_y = cy;

            // Get Work Area for screen boundaries clamping
            let mut work_area: RECT = std::mem::zeroed();
            SystemParametersInfoW(
                SPI_GETWORKAREA,
                0,
                &mut work_area as *mut RECT as *mut _,
                0,
            );

            let max_right = if work_area.right > 0 { work_area.right } else { 1920 };
            let max_bottom = if work_area.bottom > 0 { work_area.bottom } else { 1080 };
            let min_left = work_area.left;
            let min_top = work_area.top;

            // Detect actual Taskbar bounding box to ensure popup never overlaps taskbar
            let mut taskbar_top = max_bottom;
            let h_taskbar = FindWindowW(windows_sys::w!("Shell_TrayWnd"), std::ptr::null());
            if h_taskbar != std::ptr::null_mut() {
                let mut rc_tb: RECT = std::mem::zeroed();
                GetWindowRect(h_taskbar, &mut rc_tb);
                if rc_tb.top > 0 && rc_tb.top < max_bottom {
                    taskbar_top = rc_tb.top;
                }
            }

            let effective_bottom = taskbar_top.min(max_bottom);

            let fg = GetForegroundWindow();
            let flyout_rect = detect_start_or_search_flyout(fg);

            if let Some(rc_flyout) = flyout_rect {
                // Smart Dodge Start Menu / Search Flyout:
                // Automatically place Candidate Window at the nearest clear spot outside the Flyout boundary
                let flyout_mid_y = (rc_flyout.top + rc_flyout.bottom) / 2;
                let is_typing_near_top = cy < flyout_mid_y;

                if is_typing_near_top {
                    // Scenario 1 (User Image 2): Typing at the top of the Start Menu
                    // Priority 1: Directly ABOVE the top edge of the Start Menu
                    if rc_flyout.top - height - 12 >= min_top + 4 {
                        final_y = rc_flyout.top - height - 12;
                        final_x = cx.clamp(min_left + 8, max_right - width - 8);
                        if final_x < rc_flyout.left {
                            final_x = (rc_flyout.left + 12).min(max_right - width - 8);
                        } else if final_x + width > rc_flyout.right {
                            final_x = (rc_flyout.right - width - 12).max(min_left + 8);
                        }
                    }
                    // Priority 2: To the LEFT of the Start Menu near top (User Image 2 Option 2)
                    else if rc_flyout.left - width - 12 >= min_left + 8 {
                        final_x = rc_flyout.left - width - 12;
                        final_y = (rc_flyout.top + 8).clamp(min_top + 8, effective_bottom - height - 8);
                    }
                    // Priority 3: To the RIGHT of the Start Menu near top
                    else if rc_flyout.right + 12 + width <= max_right - 8 {
                        final_x = rc_flyout.right + 12;
                        final_y = (rc_flyout.top + 8).clamp(min_top + 8, effective_bottom - height - 8);
                    }
                    // Fallback
                    else {
                        final_x = min_left + 8;
                        final_y = min_top + 8;
                    }
                } else {
                    // Scenario 2 (User Image 1): Typing near the bottom (Taskbar Search Box or bottom of Start Menu)
                    // Priority 1: To the LEFT of the Start Menu, directly above the Taskbar (User Image 1 red box)
                    if rc_flyout.left - width - 12 >= min_left + 8 {
                        final_x = rc_flyout.left - width - 12;
                        final_y = effective_bottom - height - 8;
                    }
                    // Priority 2: To the RIGHT of the Start Menu, directly above the Taskbar
                    else if rc_flyout.right + 12 + width <= max_right - 8 {
                        final_x = rc_flyout.right + 12;
                        final_y = effective_bottom - height - 8;
                    }
                    // Priority 3: ABOVE the Start Menu
                    else if rc_flyout.top - height - 12 >= min_top + 4 {
                        final_y = rc_flyout.top - height - 12;
                        final_x = cx.clamp(min_left + 8, max_right - width - 8);
                    }
                    // Fallback
                    else {
                        final_x = min_left + 8;
                        final_y = effective_bottom - height - 8;
                    }
                }
            } else {
                // Normal Application: (Word, Excel, Notepad, Chrome, Edge, Zalo, Discord, IDE...)
                // If popup would overflow bottom of screen or touch taskbar, place it cleanly ABOVE the caret/cursor/taskbar
                if final_y + height > effective_bottom - 4 || cy >= effective_bottom - 50 {
                    let candidate_y_above_cursor = (cy - height - 30).min(effective_bottom - height - 8);
                    final_y = candidate_y_above_cursor.max(min_top + 8);
                }

                // Clamp horizontal boundaries
                if final_x + width > max_right - 8 {
                    final_x = (max_right - width - 8).max(min_left + 8);
                }
                if final_x < min_left + 8 {
                    final_x = min_left + 8;
                }
            }

            ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            SetWindowPos(
                hwnd,
                -1 as isize as HWND, // HWND_TOPMOST
                final_x,
                final_y,
                width,
                height,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
            BringWindowToTop(hwnd);
            InvalidateRect(hwnd, std::ptr::null(), 1);
        }
    }
}


pub fn hide_candidate_popup() {
    {
        let mut state = CANDIDATE_UI_STATE.lock();
        state.visible = false;
        state.preedit.clear();
        state.candidates.clear();
        state.candidate_rects.clear();
    }

    let hwnd = CAND_HWND.load(Ordering::SeqCst) as HWND;
    if hwnd != std::ptr::null_mut() {
        unsafe {
            ShowWindow(hwnd, SW_HIDE);
        }
    }
}

unsafe extern "system" fn cand_wnd_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_MOUSEACTIVATE => MA_NOACTIVATE,
        WM_SETCURSOR => {
            let mut pt: POINT = std::mem::zeroed();
            GetCursorPos(&mut pt);
            ScreenToClient(hwnd, &mut pt);

            let state = CANDIDATE_UI_STATE.lock();
            let is_candidate_hover = state.candidate_rects.iter().any(|(_, r)| {
                pt.x >= r.left && pt.x <= r.right && pt.y >= r.top && pt.y <= r.bottom
            });
            let is_prev_hover = pt.x >= state.prev_page_rect.left && pt.x <= state.prev_page_rect.right
                && pt.y >= state.prev_page_rect.top && pt.y <= state.prev_page_rect.bottom;
            let is_next_hover = pt.x >= state.next_page_rect.left && pt.x <= state.next_page_rect.right
                && pt.y >= state.next_page_rect.top && pt.y <= state.next_page_rect.bottom;

            if is_candidate_hover || is_prev_hover || is_next_hover {
                SetCursor(LoadCursorW(std::ptr::null_mut(), IDC_HAND));
            } else {
                SetCursor(LoadCursorW(std::ptr::null_mut(), IDC_ARROW));
            }
            1
        }
        WM_MOUSEMOVE => {
            // Trigger repaint on hover if needed
            0
        }
        WM_LBUTTONDOWN => {
            let mouse_x = (l_param & 0xFFFF) as i16 as i32;
            let mouse_y = ((l_param >> 16) & 0xFFFF) as i16 as i32;

            let (selected_cand, page_action) = {
                let state = CANDIDATE_UI_STATE.lock();
                let mut selected = None;
                let mut action = 0; // -1: prev, 1: next

                // Check prev button
                if mouse_x >= state.prev_page_rect.left && mouse_x <= state.prev_page_rect.right
                    && mouse_y >= state.prev_page_rect.top && mouse_y <= state.prev_page_rect.bottom
                {
                    action = -1;
                } else if mouse_x >= state.next_page_rect.left && mouse_x <= state.next_page_rect.right
                    && mouse_y >= state.next_page_rect.top && mouse_y <= state.next_page_rect.bottom
                {
                    action = 1;
                } else {
                    for &(idx, ref r) in &state.candidate_rects {
                        if mouse_x >= r.left && mouse_x <= r.right && mouse_y >= r.top && mouse_y <= r.bottom {
                            if idx < state.candidates.len() {
                                selected = Some(state.candidates[idx].clone());
                            }
                            break;
                        }
                    }
                }
                (selected, action)
            };

            if let Some(cand) = selected_cand {
                let mut hook_state = crate::engine::hook::GLOBAL_STATE.lock();
                let raw_candidate_text = cand.text.clone();
                let commit_text = if hook_state.variant == "Traditional" {
                    convert_s2t(&raw_candidate_text)
                } else {
                    raw_candidate_text.clone()
                };
                let raw_pinyin = hook_state.cn_raw_buffer.clone();
                let clean_pinyin = raw_pinyin.replace('\'', "").to_lowercase();

                // 1. Luôn lưu lựa chọn của người dùng cho cụm pinyin hiện tại vào User Dict!
                {
                    let mut user_dict = crate::engine::hook::USER_DICT.lock();
                    user_dict.add_phrase(&clean_pinyin, &raw_candidate_text);

                    // 2. Học cụm từ liên tiếp (chain): nếu trước đó cũng vừa commit tiếng Trung, ghép lại và lưu
                    if !hook_state.cn_prev_committed_text.is_empty() {
                        let combined_text = format!("{}{}", hook_state.cn_prev_committed_text, raw_candidate_text);
                        let combined_pinyin = format!("{}{}", hook_state.cn_prev_committed_pinyin, clean_pinyin);
                        user_dict.add_phrase(&combined_pinyin, &combined_text);
                    }
                    user_dict.save();
                }

                // Cập nhật prev commit
                hook_state.cn_prev_committed_text = raw_candidate_text.clone();
                hook_state.cn_prev_committed_pinyin = clean_pinyin;

                send_unicode_string(&commit_text);
                hook_state.cn_raw_buffer.clear();
                hook_state.cn_candidates.clear();
                hook_state.current_page = 0;
                drop(hook_state);
                hide_candidate_popup();
            } else if page_action != 0 {
                let mut hook_state = crate::engine::hook::GLOBAL_STATE.lock();
                let total_pages = (hook_state.cn_candidates.len() + hook_state.page_size - 1) / hook_state.page_size;
                if page_action == 1 && total_pages > 0 && hook_state.current_page + 1 < total_pages {
                    hook_state.current_page += 1;
                    show_candidate_popup(
                        &hook_state.cn_raw_buffer,
                        &hook_state.cn_candidates,
                        hook_state.current_page,
                        hook_state.page_size,
                        &hook_state.variant,
                    );
                } else if page_action == -1 && hook_state.current_page > 0 {
                    hook_state.current_page -= 1;
                    show_candidate_popup(
                        &hook_state.cn_raw_buffer,
                        &hook_state.cn_candidates,
                        hook_state.current_page,
                        hook_state.page_size,
                        &hook_state.variant,
                    );
                }
            }
            0
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            let mut state = CANDIDATE_UI_STATE.lock();
            state.candidate_rects.clear();

            let fs = (state.font_size as i32).max(12);
            let preedit_fs = (fs * 14 / 17).max(12);
            let page_fs = (fs * 12 / 17).max(11);
            let cand_fs = fs;

            let is_dark = is_dark_mode_active(&state.theme);

            // Palette configuration (BGR format for GDI)
            let (
                bg_color,
                border_color,
                preedit_color,
                page_active_color,
                page_inactive_color,
                page_text_color,
                cand1_color,
                cand_other_color,
                badge_color,
            ) = if is_dark {
                (
                    0x002A170F, // Background: Slate 900 (RGB 15, 23, 42 -> BGR 0x002A170F)
                    0x00F6823B, // Border: Blue 500 (RGB 59, 130, 246 -> BGR 0x00F6823B)
                    0x00FA8560, // Preedit: Sky Blue (RGB 96, 165, 250 -> BGR 0x00FA8560)
                    0x00FA8560, // Page Active: Sky Blue
                    0x0064748B, // Page Inactive: Slate 600
                    0x00CBD5E1, // Page Text: Slate 300
                    0x00F8BD38, // Cand #1: Cyan (RGB 56, 189, 248 -> BGR 0x00F8BD38)
                    0x00F9F5F1, // Other Cands: Off-white (RGB 241, 245, 249 -> BGR 0x00F9F5F1)
                    0x0024BFFA, // Badge: Amber
                )
            } else {
                (
                    0x00FFFFFF, // Background: White
                    0x00EB6325, // Border: Primary Blue #2563eb
                    0x00EB6325, // Preedit: Primary Blue
                    0x00EB6325, // Page Active: Primary Blue
                    0x00CBD5E1, // Page Inactive: Slate 300
                    0x0064748B, // Page Text: Slate Muted
                    0x00D84E1D, // Cand #1: Primary Blue #1d4ed8
                    0x003B291E, // Other Cands: Dark Slate #1e293b
                    0x000677D9, // Badge: Amber
                )
            };

            // Background & Border
            let bg_brush = CreateSolidBrush(bg_color);
            let border_pen = CreatePen(PS_SOLID as i32, 1, border_color);
            let old_pen = SelectObject(hdc, border_pen);
            let old_brush = SelectObject(hdc, bg_brush);

            let mut rect: RECT = std::mem::zeroed();
            windows_sys::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut rect);
            RoundRect(hdc, rect.left, rect.top, rect.right, rect.bottom, 10, 10);

            SetBkMode(hdc, TRANSPARENT as i32);

            // Fonts (Negative height for exact character height matching)
            let h_font_preedit = CreateFontW(
                -preedit_fs, 0, 0, 0, FW_BOLD as i32, 0, 0, 0, 1, 0, 0, 0, 0,
                windows_sys::w!("Segoe UI"),
            );
            let h_font_page = CreateFontW(
                -page_fs, 0, 0, 0, FW_NORMAL as i32, 0, 0, 0, 1, 0, 0, 0, 0,
                windows_sys::w!("Segoe UI"),
            );
            let h_font_cand = CreateFontW(
                -cand_fs, 0, 0, 0, FW_NORMAL as i32, 0, 0, 0, 1, 0, 0, 0, 0,
                windows_sys::w!("Microsoft YaHei"),
            );

            let header_top = 4;
            let header_bottom = header_top + (fs * 15 / 17).max(18);

            // 1. Pagination Indicator & Buttons (e.g. "◀ 1/8 ▶")
            SelectObject(hdc, h_font_page);
            let total_pages = if state.candidates.is_empty() { 1 } else { (state.candidates.len() + state.page_size - 1) / state.page_size };
            let page_text = format!("{}/{}", state.current_page + 1, total_pages);
            
            // Prev button ◀
            let prev_str = "◀";
            let prev_utf16: Vec<u16> = prev_str.encode_utf16().collect();
            state.prev_page_rect = RECT {
                left: rect.right - 92,
                top: header_top,
                right: rect.right - 80,
                bottom: header_bottom,
            };
            SetTextColor(hdc, if state.current_page > 0 { page_active_color } else { page_inactive_color });
            DrawTextW(
                hdc,
                prev_utf16.as_ptr(),
                prev_utf16.len() as i32,
                &mut state.prev_page_rect,
                DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
            );

            // Page text
            let page_utf16: Vec<u16> = page_text.encode_utf16().collect();
            let mut page_rect = RECT {
                left: rect.right - 78,
                top: header_top,
                right: rect.right - 50,
                bottom: header_bottom,
            };
            SetTextColor(hdc, page_text_color);
            DrawTextW(
                hdc,
                page_utf16.as_ptr(),
                page_utf16.len() as i32,
                &mut page_rect,
                DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
            );

            // Next button ▶
            let next_str = "▶";
            let next_utf16: Vec<u16> = next_str.encode_utf16().collect();
            state.next_page_rect = RECT {
                left: rect.right - 48,
                top: header_top,
                right: rect.right - 36,
                bottom: header_bottom,
            };
            SetTextColor(hdc, if state.current_page + 1 < total_pages { page_active_color } else { page_inactive_color });
            DrawTextW(
                hdc,
                next_utf16.as_ptr(),
                next_utf16.len() as i32,
                &mut state.next_page_rect,
                DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
            );

            // 2. Badge [简] or [繁]
            let badge_text = if state.variant == "Traditional" { "[繁]" } else { "[简]" };
            let badge_utf16: Vec<u16> = badge_text.encode_utf16().collect();
            SetTextColor(hdc, badge_color);
            let mut badge_rect = RECT {
                left: rect.right - 34,
                top: header_top,
                right: rect.right - 8,
                bottom: header_bottom,
            };
            DrawTextW(
                hdc,
                badge_utf16.as_ptr(),
                badge_utf16.len() as i32,
                &mut badge_rect,
                DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
            );

            // 3. Preedit text (e.g. "qua", "quan")
            SelectObject(hdc, h_font_preedit);
            SetTextColor(hdc, preedit_color);
            let preedit_utf16: Vec<u16> = state.preedit.encode_utf16().collect();
            let mut pre_rect = RECT {
                left: 10,
                top: header_top,
                right: state.prev_page_rect.left - 6,
                bottom: header_bottom,
            };
            DrawTextW(
                hdc,
                preedit_utf16.as_ptr(),
                preedit_utf16.len() as i32,
                &mut pre_rect,
                DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
            );

            // 4. Candidates list
            SelectObject(hdc, h_font_cand);
            let start = state.current_page * state.page_size;
            let end = (start + state.page_size).min(state.candidates.len());
            let mut cur_x = 10;
            let mut local_rects: Vec<(usize, RECT)> = Vec::new();

            let cand_top = header_bottom + 2;
            let cand_bottom = cand_top + cand_fs + 8;
            let cand_item_gap = (fs * 4 / 10).max(6); // Compact gap between items (half of previous)

            for i in start..end {
                let cand_text = if state.variant == "Traditional" {
                    convert_s2t(&state.candidates[i].text)
                } else {
                    state.candidates[i].text.clone()
                };
                let num = (i - start) + 1;
                let cand_str = format!("{}.{}", num, cand_text);
                let cand_utf16: Vec<u16> = cand_str.encode_utf16().collect();

                let mut size: SIZE = std::mem::zeroed();
                GetTextExtentPoint32W(hdc, cand_utf16.as_ptr(), cand_utf16.len() as i32, &mut size);
                let item_text_width = size.cx;

                // Highlight candidate 1, others in secondary color
                if i == start {
                    SetTextColor(hdc, cand1_color);
                } else {
                    SetTextColor(hdc, cand_other_color);
                }

                let mut cand_rect = RECT {
                    left: cur_x,
                    top: cand_top,
                    right: cur_x + item_text_width,
                    bottom: cand_bottom,
                };
                DrawTextW(
                    hdc,
                    cand_utf16.as_ptr(),
                    cand_utf16.len() as i32,
                    &mut cand_rect,
                    DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
                );

                // Save bounding box for mouse click & hover (covers item + half gap)
                let click_rect = RECT {
                    left: cur_x,
                    top: cand_top,
                    right: cur_x + item_text_width + cand_item_gap,
                    bottom: cand_bottom,
                };
                local_rects.push((i, click_rect));

                cur_x += item_text_width + cand_item_gap;
            }
            state.candidate_rects = local_rects;

            // Cleanup GDI objects
            DeleteObject(h_font_preedit);
            DeleteObject(h_font_page);
            DeleteObject(h_font_cand);
            SelectObject(hdc, old_pen);
            SelectObject(hdc, old_brush);
            DeleteObject(bg_brush);
            DeleteObject(border_pen);

            EndPaint(hwnd, &ps);
            0
        }
        _ => DefWindowProcW(hwnd, msg, w_param, l_param),
    }
}

pub fn start_candidate_window_thread() {
    std::thread::spawn(|| unsafe {
        let class_name = windows_sys::w!("VCKeys_CandWndClass");
        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(cand_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: 0 as isize as _,
            hIcon: 0 as isize as _,
            hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
            hbrBackground: 0 as isize as _,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name,
            hIconSm: 0 as isize as _,
        };

        RegisterClassExW(&wnd_class);

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class_name,
            windows_sys::w!("VCKeys Candidate"),
            WS_POPUP,
            100,
            100,
            300,
            56,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0 as isize as _,
            std::ptr::null(),
        );

        if hwnd != std::ptr::null_mut() {
            CAND_HWND.store(hwnd as isize, Ordering::SeqCst);
            let mut msg = std::mem::zeroed();
            while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&mut msg);
                DispatchMessageW(&mut msg);
            }
        }
    });
}
