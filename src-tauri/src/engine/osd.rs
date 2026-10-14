use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::Arc;
use parking_lot::Mutex;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint,
    InvalidateRect, RoundRect, SelectObject, SetBkMode, SetTextColor, DT_CENTER, DT_NOPREFIX,
    DT_SINGLELINE, DT_VCENTER, FW_BOLD, PAINTSTRUCT, PS_SOLID, TRANSPARENT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW,
    KillTimer, PostMessageW, RegisterClassExW, SetTimer, SetWindowPos, ShowWindow, SystemParametersInfoW,
    TranslateMessage, CS_HREDRAW, CS_VREDRAW, SPI_GETWORKAREA, SWP_HIDEWINDOW,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SW_HIDE, SW_SHOWNOACTIVATE,
    WM_PAINT, WM_TIMER, WNDCLASSEXW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

pub struct OsdState {
    pub text: String,
    pub bg_color: u32,      // 0x00RRGGBB format in GDI (BGR: 0x00BBGGRR)
    pub tag: String,
}

impl Default for OsdState {
    fn default() -> Self {
        Self {
            text: "VCKeys: TIẾNG VIỆT (Telex)".to_string(),
            bg_color: 0x00481DE1, // #e11d48 (BGR: 0x481DE1)
            tag: "VN".to_string(),
        }
    }
}

lazy_static::lazy_static! {
    pub static ref OSD_STATE: Arc<Mutex<OsdState>> = Arc::new(Mutex::new(OsdState::default()));
    pub static ref OSD_HWND: AtomicIsize = AtomicIsize::new(0);
}

const TIMER_ID_HIDE: usize = 999;
const WM_USER_SHOW_OSD: u32 = 0x0400 + 101;

pub fn show_osd_notification(mode: &str) {
    let (text, bgr_color, tag) = match mode {
        "Chinese" => ("VCKeys: TIẾNG TRUNG (Pinyin)", 0x00EB6325, "CN"), // #2563eb Blue
        "English" => ("VCKeys: TIẾNG ANH (English)", 0x00695547, "EN"),   // #475569 Slate
        _ => ("VCKeys: TIẾNG VIỆT (Telex)", 0x003530E5, "VN"),           // #e53035 Red
    };
    show_osd_custom(tag, text, bgr_color);
}

pub fn show_osd_custom(tag: &str, text: &str, bgr_color: u32) {
    {
        let mut state = OSD_STATE.lock();
        state.text = text.to_string();
        state.bg_color = bgr_color;
        state.tag = tag.to_string();
    }

    let hwnd = OSD_HWND.load(Ordering::SeqCst) as HWND;
    if hwnd != std::ptr::null_mut() {
        unsafe {
            PostMessageW(hwnd, WM_USER_SHOW_OSD, 0, 0);
        }
    }
}

unsafe extern "system" fn osd_wnd_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_USER_SHOW_OSD => {
            let mut work_area: RECT = std::mem::zeroed();
            SystemParametersInfoW(
                SPI_GETWORKAREA,
                0,
                &mut work_area as *mut RECT as _,
                0,
            );

            let width = 230;
            let height = 34;
            let x = (work_area.right - width - 16).max(0);
            let y = (work_area.bottom - height - 12).max(0);

            SetWindowPos(
                hwnd,
                -1 as isize as HWND, // HWND_TOPMOST
                x,
                y,
                width,
                height,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
            InvalidateRect(hwnd, std::ptr::null(), 1);
            ShowWindow(hwnd, SW_SHOWNOACTIVATE);

            // Reset hide timer to 1300ms on the window thread
            KillTimer(hwnd, TIMER_ID_HIDE);
            SetTimer(hwnd, TIMER_ID_HIDE, 1300, None);
            0
        }
        WM_TIMER => {
            if w_param == TIMER_ID_HIDE {
                KillTimer(hwnd, TIMER_ID_HIDE);
                ShowWindow(hwnd, SW_HIDE);
                SetWindowPos(
                    hwnd,
                    0 as _,
                    -10000,
                    -10000,
                    0,
                    0,
                    SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_HIDEWINDOW,
                );
            }
            0
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            let state = OSD_STATE.lock();

            let mut rect: RECT = std::mem::zeroed();
            GetClientRect(hwnd, &mut rect);

            // Background brush
            let bg_brush = CreateSolidBrush(state.bg_color);
            let border_pen = CreatePen(PS_SOLID as i32, 1, state.bg_color);
            let old_pen = SelectObject(hdc, border_pen);
            let old_brush = SelectObject(hdc, bg_brush);

            // Rounded rectangle
            RoundRect(hdc, rect.left, rect.top, rect.right, rect.bottom, 8, 8);

            SetBkMode(hdc, TRANSPARENT as i32);

            let h_font_tag = CreateFontW(
                12, 0, 0, 0, FW_BOLD as i32, 0, 0, 0, 1, 0, 0, 0, 0,
                windows_sys::w!("Segoe UI"),
            );
            let h_font_text = CreateFontW(
                14, 0, 0, 0, FW_BOLD as i32, 0, 0, 0, 1, 0, 0, 0, 0,
                windows_sys::w!("Segoe UI"),
            );

            // Draw Tag (e.g. "VN", "CN", "EN")
            SelectObject(hdc, h_font_tag);
            SetTextColor(hdc, 0x00FFFFFF);
            let tag_utf16: Vec<u16> = state.tag.encode_utf16().collect();
            let mut tag_rect = RECT {
                left: 10,
                top: 7,
                right: 36,
                bottom: 27,
            };
            DrawTextW(
                hdc,
                tag_utf16.as_ptr(),
                tag_utf16.len() as i32,
                &mut tag_rect,
                DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
            );

            // Draw Message Text (e.g. "VCKey: TIẾNG VIỆT (Telex)")
            SelectObject(hdc, h_font_text);
            SetTextColor(hdc, 0x00FFFFFF);
            let text_utf16: Vec<u16> = state.text.encode_utf16().collect();
            let mut text_rect = RECT {
                left: 40,
                top: 0,
                right: rect.right - 10,
                bottom: rect.bottom,
            };
            DrawTextW(
                hdc,
                text_utf16.as_ptr(),
                text_utf16.len() as i32,
                &mut text_rect,
                DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
            );

            // Clean up GDI objects
            DeleteObject(h_font_tag);
            DeleteObject(h_font_text);
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

pub fn start_osd_window_thread() {
    std::thread::spawn(|| unsafe {
        let class_name = windows_sys::w!("VCKeys_OsdWndClass");
        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(osd_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: 0 as isize as _,
            hIcon: 0 as isize as _,
            hCursor: 0 as isize as _,
            hbrBackground: 0 as isize as _,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name,
            hIconSm: 0 as isize as _,
        };

        RegisterClassExW(&wnd_class);

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class_name,
            windows_sys::w!("VCKeys OSD"),
            WS_POPUP,
            100,
            100,
            230,
            34,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0 as isize as _,
            std::ptr::null(),
        );

        if hwnd != std::ptr::null_mut() {
            OSD_HWND.store(hwnd as isize, Ordering::SeqCst);
            let mut msg = std::mem::zeroed();
            while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&mut msg);
                DispatchMessageW(&mut msg);
            }
        }
    });
}
