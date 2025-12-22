use parking_lot::Mutex;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use std::sync::Arc;
use once_cell::sync::OnceCell;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM, POINT, SIZE, RECT, COLORREF};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Gdi::{PAINTSTRUCT, BeginPaint, EndPaint};
// rely on wildcard imports for GetClientRect/PostMessage/UpdateWindow equivalents
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::Foundation::GetLastError;
use windows::Win32::Graphics::Gdi::{HDC, HGDIOBJ, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CreateCompatibleDC, CreateDIBSection, SelectObject, DeleteObject, DeleteDC, DrawTextW, DT_LEFT, DT_WORDBREAK, DT_NOPREFIX, DT_CALCRECT, SetBkMode, SetTextColor, DIB_RGB_COLORS, GetDC, ReleaseDC, BLENDFUNCTION, FillRect, CreateSolidBrush, CreateFontW, HFONT};

static SUGGESTION_CELL: OnceCell<Arc<Mutex<String>>> = OnceCell::new();
static mut GLOBAL_HWND: HWND = HWND(0);
static mut LAST_X: i32 = -1;
static mut LAST_Y: i32 = -1;
static mut LAYERED_WINDOW_CREATED: bool = true;  // Will be set from main

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

fn clamp(v: i32, lo: i32, hi: i32) -> i32 { if v < lo { lo } else if v > hi { hi } else { v } }

// Export window proc for main thread to use
pub unsafe extern "system" fn window_proc_export(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    window_proc(hwnd, msg, wparam, lparam)
}

// Set HWND created from main thread
pub fn set_overlay_hwnd(hwnd: HWND) {
    unsafe {
        GLOBAL_HWND = hwnd;
        crate::logger::log(&format!("overlay: set_overlay_hwnd called with hwnd.0={}", hwnd.0));
    }
}

pub fn run_overlay(suggestion: Arc<Mutex<String>>) -> anyhow::Result<()> {
    crate::logger::log("overlay: run_overlay starting (using pre-created window)");
    let _ = SUGGESTION_CELL.set(suggestion);
    
    unsafe {
        crate::logger::log("overlay: creating window (self-managed)");

        let hinstance = GetModuleHandleW(None).unwrap_or_default();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let class_name_str = format!("UCO_DEBUG_{}", timestamp);
        let class_name = to_wide(&class_name_str);

        let _ = UnregisterClassW(PCWSTR(class_name.as_ptr()), hinstance);

        let wc = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: hinstance,
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        let atom = RegisterClassW(&wc);
        crate::logger::log(&format!("overlay: RegisterClassW returned {}", atom));
        if atom == 0 {
            crate::logger::log(&format!("overlay: RegisterClassW failed GetLastError={}", GetLastError().0));
            return Ok(());
        }

        let ex_style = WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE;
        let hwnd = CreateWindowExW(
            ex_style,
            PCWSTR(class_name.as_ptr()),
            None,
            WS_POPUP,
            100,
            100,
            600,
            90,
            HWND(0),
            None,
            hinstance,
            None,
        );

        crate::logger::log(&format!("overlay: CreateWindowExW returned hwnd.0={}", hwnd.0));
        if hwnd.0 == 0 {
            crate::logger::log(&format!("overlay: CreateWindowExW failed GetLastError={}", GetLastError().0));
            return Ok(());
        }

        GLOBAL_HWND = hwnd;
        crate::logger::log(&format!("overlay: Using HWND={} (self-created)", GLOBAL_HWND.0));

        // Show but do not activate (no focus steal)
        ShowWindow(GLOBAL_HWND, SW_SHOW);

        // Force an initial position onscreen for debug (top-left) and keep it topmost
        SetWindowPos(
            GLOBAL_HWND,
            HWND_TOPMOST,
            100,
            100,
            0,
            0,
            SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );

        // Initial render using layered renderer
            crate::logger::log("overlay: calling render_layered_overlay (initial)");
        render_layered_overlay();

        let mut msg = MSG::default();
        crate::logger::log("overlay: entering message loop");
        while GetMessageW(&mut msg, HWND(0), 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        crate::logger::log("overlay: message loop exited");

        Ok(())
    }
}

unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            crate::logger::log("overlay::WM_CREATE received");
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_WINDOWPOSCHANGED => {
            // Re-render after moves/resizes
            if SUGGESTION_CELL.get().is_some() {
                render_layered_overlay();
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_PAINT => {
            // Paint for non-layered fallback windows
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            if hdc.0 != 0 {
                // Fill background with a light gray to make overlay visible
                let brush = CreateSolidBrush(COLORREF(0x00F0F0F0));
                FillRect(hdc, &ps.rcPaint, brush);

                // Draw suggestion text
                let mut client_rc = RECT::default();
                let _ = GetClientRect(hwnd, &mut client_rc);
                let text = if let Some(s) = SUGGESTION_CELL.get() {
                    let lock = s.lock();
                    lock.clone()
                } else { String::new() };
                if !text.is_empty() {
                    SetBkMode(hdc, windows::Win32::Graphics::Gdi::BACKGROUND_MODE(1));
                    SetTextColor(hdc, COLORREF(0x00000000));
                    let mut wide = to_wide(&text.chars().take(200).collect::<String>());
                    let wide_len = wide.len();
                    if wide_len > 0 {
                        let text_slice = &mut wide[..wide_len-1];
                        DrawTextW(hdc, text_slice, &mut client_rc, DT_LEFT | DT_WORDBREAK | DT_NOPREFIX);
                    }
                }

                // cleanup
                DeleteObject(HGDIOBJ(brush.0 as isize));
            }
            EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_APP => {
            // custom app message -> re-render
            if SUGGESTION_CELL.get().is_some() {
                render_layered_overlay();
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn measure_text(hdc: HDC, text: &str) -> (i32, i32) {
    unsafe {
        let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        let mut wide = to_wide(text);
        if wide.len() > 1 {
            let len = wide.len();
            let _ = DrawTextW(hdc, &mut wide[..len-1], &mut rect, DT_LEFT | DT_WORDBREAK | DT_NOPREFIX | DT_CALCRECT);
        }
        (rect.right - rect.left, rect.bottom - rect.top)
    }
}

// Core renderer: creates a 32bpp ARGB DIB, fills glass background, draws text, and updates layered window
fn render_layered_overlay() {
    unsafe {
        if GLOBAL_HWND.0 == 0 { return; }

        // Get current suggestion text
        let text = if let Some(s) = SUGGESTION_CELL.get() {
            let lock = s.lock();
            lock.clone()
        } else { String::new() };
        if text.is_empty() { return; }

        // Get screen DC and memory DC
        let screen_dc = GetDC(HWND(0));
        if screen_dc.0 == 0 { crate::logger::log("overlay: screen_dc=0"); return; }
        let mem_dc = CreateCompatibleDC(screen_dc);
        if mem_dc.0 == 0 { crate::logger::log("overlay: mem_dc=0"); ReleaseDC(HWND(0), screen_dc); return; }

        // Use Segoe UI with ClearType for readability
        let font: HFONT = CreateFontW(
            -18, 0, 0, 0, 400, 0, 0, 0,
            1, 4, 0, 5, 0,
            PCWSTR(to_wide("Segoe UI").as_ptr()),
        );
        let old_font = SelectObject(mem_dc, HGDIOBJ(font.0 as isize));

        // Measure text with max width and padding
        let max_width = 520;
        let pad_x = 24;
        let pad_y = 18;
        let (tw, th) = {
            let mut rect = RECT { left: 0, top: 0, right: max_width, bottom: 0 };
            let mut wide = to_wide(&text.chars().take(400).collect::<String>());
            if wide.len() > 1 {
                let len = wide.len();
                let _ = DrawTextW(mem_dc, &mut wide[..len-1], &mut rect, DT_LEFT | DT_WORDBREAK | DT_NOPREFIX | DT_CALCRECT);
            }
            (rect.right - rect.left, rect.bottom - rect.top)
        };
        let width = clamp(tw + pad_x * 2, 180, max_width + pad_x * 2);
        let height = clamp(th + pad_y * 2, 40, 240);

        // Prepare DIB info (32bpp ARGB, top-down)
        let mut bmi = BITMAPINFO::default();
        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = width;
        bmi.bmiHeader.biHeight = -height; // top-down
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB.0 as u32;

        let mut pv_bits: *mut std::ffi::c_void = null_mut();
        let hbitmap = match CreateDIBSection(mem_dc, &bmi, DIB_RGB_COLORS, &mut pv_bits, None, 0) {
            Ok(h) => h,
            Err(_) => {
                DeleteDC(mem_dc);
                ReleaseDC(HWND(0), screen_dc);
                return;
            }
        };
        if hbitmap.0 == 0 || pv_bits.is_null() {
            DeleteDC(mem_dc);
            ReleaseDC(HWND(0), screen_dc);
            return;
        }

        // Glassmorphic styling: semi-transparent dark background with subtle rounded effect
        let alpha: u8 = 200;
        let bg_r: u8 = 30;
        let bg_g: u8 = 30;
        let bg_b: u8 = 38;
        let border_radius: i32 = 8;

        let pixels = std::slice::from_raw_parts_mut(pv_bits as *mut u8, (width * height * 4) as usize);
        for y in 0..height {
            for x in 0..width {
                let i = ((y * width + x) * 4) as usize;
                
                // Soft rounded corners (approximate)
                let dist_to_corner = if x < border_radius && y < border_radius {
                    let dx = border_radius - x;
                    let dy = border_radius - y;
                    ((dx*dx + dy*dy) as f32).sqrt()
                } else if x >= width - border_radius && y < border_radius {
                    let dx = x - (width - border_radius - 1);
                    let dy = border_radius - y;
                    ((dx*dx + dy*dy) as f32).sqrt()
                } else if x < border_radius && y >= height - border_radius {
                    let dx = border_radius - x;
                    let dy = y - (height - border_radius - 1);
                    ((dx*dx + dy*dy) as f32).sqrt()
                } else if x >= width - border_radius && y >= height - border_radius {
                    let dx = x - (width - border_radius - 1);
                    let dy = y - (height - border_radius - 1);
                    ((dx*dx + dy*dy) as f32).sqrt()
                } else {
                    0.0
                };
                
                let corner_alpha = if dist_to_corner > border_radius as f32 {
                    0
                } else if dist_to_corner > (border_radius - 2) as f32 {
                    ((1.0 - (dist_to_corner - (border_radius - 2) as f32) / 2.0) * alpha as f32) as u8
                } else {
                    alpha
                };
                
                let final_alpha = if dist_to_corner == 0.0 { alpha } else { corner_alpha };
                let final_r = ((bg_r as u16 * final_alpha as u16) / 255) as u8;
                let final_g = ((bg_g as u16 * final_alpha as u16) / 255) as u8;
                let final_b = ((bg_b as u16 * final_alpha as u16) / 255) as u8;
                
                pixels[i + 0] = final_b;
                pixels[i + 1] = final_g;
                pixels[i + 2] = final_r;
                pixels[i + 3] = final_alpha;
            }
        }

        // Select bitmap into DC
        let old_bmp = SelectObject(mem_dc, HGDIOBJ(hbitmap.0));

        // Set text properties (transparent background, light gray text)
        SetBkMode(mem_dc, windows::Win32::Graphics::Gdi::BACKGROUND_MODE(1));
        SetTextColor(mem_dc, COLORREF(0x00B0B0B0));

        // Suggestion text already retrieved above

        // Draw main suggestion (wrap/truncate)
        let main = text.chars().take(200).collect::<String>();
        let mut wide = to_wide(&main);
        let text_len = wide.len();
        let mut rect = RECT { left: 16, top: 12, right: width - 16, bottom: height - 12 };
        if text_len > 0 {
            DrawTextW(mem_dc, &mut wide[..text_len-1], &mut rect, DT_LEFT | DT_WORDBREAK | DT_NOPREFIX);
        }

        // Clean up font selection
        SelectObject(mem_dc, old_font);

        // Resize window to match measured size before updating surface
        let pos_x = if LAST_X < 0 { 100 } else { LAST_X };
        let pos_y = if LAST_Y < 0 { 100 } else { LAST_Y + 20 };
        SetWindowPos(GLOBAL_HWND, HWND_TOPMOST, pos_x, pos_y, width, height, SWP_NOACTIVATE | SWP_SHOWWINDOW);

        // Prepare parameters for UpdateLayeredWindow
        let pt_src = POINT { x: 0, y: 0 };
        let pt_pos = POINT { x: pos_x, y: pos_y };
        let size = SIZE { cx: width, cy: height };

        // BLENDFUNCTION uses u8 fields; build manually
        let blend = BLENDFUNCTION { BlendOp: 0, BlendFlags: 0, SourceConstantAlpha: 255u8, AlphaFormat: 1u8 };

        // Update layered window with the DIB
        let _ = UpdateLayeredWindow(GLOBAL_HWND, screen_dc, Some(&pt_pos), Some(&size), mem_dc, Some(&pt_src), COLORREF(0), Some(&blend), ULW_ALPHA);

        // cleanup
        SelectObject(mem_dc, old_bmp);
        DeleteObject(HGDIOBJ(hbitmap.0));
        DeleteObject(HGDIOBJ(font.0 as isize));
        DeleteDC(mem_dc);
        ReleaseDC(HWND(0), screen_dc);
    }
}

pub struct CaretRect { pub x: i32, pub y: i32 }

pub fn update_overlay_position(x: i32, y: i32) {
    unsafe {
        if GLOBAL_HWND.0 != 0 {
            if (x - LAST_X).abs() > 2 || (y - LAST_Y).abs() > 2 {
                LAST_X = clamp(x, 0, 65535);
                LAST_Y = clamp(y, 0, 65535);
                // Move window and keep it topmost during debug
                SetWindowPos(GLOBAL_HWND, HWND_TOPMOST, LAST_X, LAST_Y + 20, 0, 0, SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW);
                // Re-render at new position
                render_layered_overlay();
            }
        }
    }
}

pub fn clear_suggestion() {
    if let Some(s) = SUGGESTION_CELL.get() {
        let mut lock = s.lock();
        *lock = String::new();
    }
    render_layered_overlay();
}

// Allow other modules to push a short status into the overlay
pub fn set_suggestion_text(text: &str) {
    if let Some(s) = SUGGESTION_CELL.get() {
        let mut lock = s.lock();
        *lock = text.to_string();
    }
    render_layered_overlay();
}

pub fn invalidate_overlay() {
    unsafe {
        if GLOBAL_HWND.0 != 0 {
            crate::logger::log(&format!("invalidate_overlay: Posting WM_APP to HWND={}", GLOBAL_HWND.0));
            // Signal the overlay to re-render via custom app message
            let result = PostMessageW(GLOBAL_HWND, WM_APP, WPARAM(0), LPARAM(0));
            crate::logger::log(&format!("invalidate_overlay: PostMessageW result={}", result.as_bool()));
        } else {
            crate::logger::log("invalidate_overlay: WARNING - GLOBAL_HWND is 0, cannot post message!");
        }
    }
}

