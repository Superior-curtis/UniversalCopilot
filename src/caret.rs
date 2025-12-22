use anyhow::Result;
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::*;

#[link(name = "user32")]
extern "system" {
    fn AttachThreadInput(idAttach: u32, idAttachTo: u32, fAttach: i32) -> i32;
    fn GetCaretPos(lpPoint: *mut POINT) -> i32;
    fn GetCursorPos(lpPoint: *mut POINT) -> i32;
}

pub struct CaretRect {
    pub x: i32,
    pub y: i32,
    pub hwnd: HWND,
}

pub fn get_caret_rect() -> Result<CaretRect> {
    unsafe {
        let hwnd_fore = GetForegroundWindow();
        println!("caret::get_caret_rect: GetForegroundWindow -> HWND={}", hwnd_fore.0);
        crate::logger::log(&format!("caret: GetForegroundWindow HWND={}", hwnd_fore.0));
        // Silent failure: no active window (screen saver, lock screen, etc)
        if hwnd_fore.0 == 0 {
            anyhow::bail!("no foreground window");
        }

        let tid = GetWindowThreadProcessId(hwnd_fore, Some(std::ptr::null_mut()));
        if tid == 0 {
            eprintln!("caret::get_caret_rect: invalid thread id");
            crate::logger::log("caret: invalid thread id");
            anyhow::bail!("invalid thread id");
        }
        
        // Try thread input attachment first for direct caret position
        let current_tid = GetCurrentThreadId();
        if AttachThreadInput(current_tid, tid, 1) != 0 {
            let mut pt = POINT { x: 0, y: 0 };
            if GetCaretPos(&mut pt) != 0 && (pt.x != 0 || pt.y != 0) {
                // Got valid caret position, convert to screen coordinates
                let mut screen_pt = pt;
                let _ = ClientToScreen(hwnd_fore, &mut screen_pt).as_bool();
                crate::logger::log(&format!("caret: AttachThreadInput success, screen_pos=({}, {})", screen_pt.x, screen_pt.y));
                println!("caret::get_caret_rect: AttachThreadInput screen_pos=({}, {})", screen_pt.x, screen_pt.y);
                let _ = AttachThreadInput(current_tid, tid, 0);
                return Ok(CaretRect { x: screen_pt.x, y: screen_pt.y, hwnd: hwnd_fore });
            }
            let _ = AttachThreadInput(current_tid, tid, 0);
        }

        // Fallback to GetGUIThreadInfo
        let mut gui = GUITHREADINFO::default();
        gui.cbSize = std::mem::size_of::<GUITHREADINFO>() as u32;
        
        // Try to get GUI thread info; fails on privileged windows (explorer, etc)
        if !GetGUIThreadInfo(tid, &mut gui).as_bool() {
            eprintln!("caret::get_caret_rect: GetGUIThreadInfo failed for tid={}", tid);
            crate::logger::log(&format!("caret: GetGUIThreadInfo failed tid={}", tid));
            // Ultimate fallback: use mouse cursor position (works for all apps)
            let mut cursor_pt = POINT { x: 0, y: 0 };
            if GetCursorPos(&mut cursor_pt) != 0 {
                crate::logger::log(&format!("caret: Using cursor fallback pos=({}, {})", cursor_pt.x, cursor_pt.y));
                println!("caret::get_caret_rect: Using cursor fallback pos=({}, {})", cursor_pt.x, cursor_pt.y);
                return Ok(CaretRect { x: cursor_pt.x, y: cursor_pt.y, hwnd: hwnd_fore });
            }
            anyhow::bail!("GetGUIThreadInfo failed and cursor fallback also failed");
        }

        // rcCaret may be empty/invalid; check before using
        if gui.rcCaret.right <= gui.rcCaret.left || gui.rcCaret.bottom <= gui.rcCaret.top {
            eprintln!("caret::get_caret_rect: invalid caret rect from GetGUIThreadInfo: {:?}", gui.rcCaret);
            crate::logger::log(&format!("caret: invalid caret rect from GetGUIThreadInfo: {:?}", gui.rcCaret));
            // Fallback to cursor position
            let mut cursor_pt = POINT { x: 0, y: 0 };
            if GetCursorPos(&mut cursor_pt) != 0 {
                crate::logger::log(&format!("caret: Using cursor fallback pos=({}, {})", cursor_pt.x, cursor_pt.y));
                println!("caret::get_caret_rect: Using cursor fallback pos=({}, {})", cursor_pt.x, cursor_pt.y);
                return Ok(CaretRect { x: cursor_pt.x, y: cursor_pt.y, hwnd: hwnd_fore });
            }
            anyhow::bail!("invalid caret rect from GetGUIThreadInfo and cursor fallback failed");
        }

        let mut left = gui.rcCaret.left;
        let mut top = gui.rcCaret.top;
        // Prefer hwndCaret; fall back to hwndFocus; fail if both invalid
        let target = if gui.hwndCaret.0 != 0 { 
            gui.hwndCaret 
        } else if gui.hwndFocus.0 != 0 {
            gui.hwndFocus 
        } else {
            anyhow::bail!("no target window for caret");
        };

        // Convert caret position to screen coordinates when possible
        if target.0 != 0 {
            let mut pt = POINT { x: left, y: top };
            // ClientToScreen returns BOOL; ignore failure and keep original coords
            let _ = ClientToScreen(target, &mut pt).as_bool();
            left = pt.x;
            top = pt.y;
        }

        println!("caret::get_caret_rect: GetGUIThreadInfo target HWND={} screen_pos=({}, {})", target.0, left, top);
        crate::logger::log(&format!("caret: GetGUIThreadInfo target HWND={} screen_pos=({}, {})", target.0, left, top));

        Ok(CaretRect { x: left, y: top, hwnd: target })
    }
}
