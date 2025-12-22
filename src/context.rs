use anyhow::Result;
use windows::Win32::Foundation::{WPARAM, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::*;
use crate::caret;

const EM_GETSEL: u32 = 0x00B0;

pub struct TextContext {
    pub before: String,
    pub after: String,
}

pub fn capture_active_context() -> Result<TextContext> {
    unsafe {
        // get caret info including hwnd
        let caret = caret::get_caret_rect()?;
        let hwnd = caret.hwnd;
        if hwnd.0 == 0 {
            anyhow::bail!("no focused control");
        }

        // get text length
        let len = SendMessageW(hwnd, WM_GETTEXTLENGTH, WPARAM(0), LPARAM(0)).0 as i32;
        let mut buf: Vec<u16> = vec![0u16; (len + 1) as usize];
        if len > 0 {
            let ret = SendMessageW(hwnd, WM_GETTEXT, WPARAM((len + 1) as usize), LPARAM(buf.as_mut_ptr() as isize));
            let _ = ret;
        }
        // convert UTF-16 buffer to Rust String (properly handling surrogates)
        let text = if let Ok(s) = String::from_utf16(&buf[..(len as usize)]) {
            s
        } else {
            String::from_utf16_lossy(&buf[..(len as usize)]).to_owned()
        };

        // attempt to get selection/caret index
        let mut start: u32 = 0;
        let mut end: u32 = 0;
        // EM_GETSEL expects pointers to start and end
        let _ = SendMessageW(hwnd, EM_GETSEL, WPARAM(&mut start as *mut _ as usize), LPARAM(&mut end as *mut _ as isize));

        // Convert UTF-16 code unit offset to Rust String char index.
        // EM_GETSEL returns offsets in UTF-16 code units, we need char boundaries.
        let char_index = utf16_offset_to_char_index(&buf[..(len as usize)], start as usize);

        let before: String = text.chars().take(char_index).collect();
        let after: String = text.chars().skip(char_index).collect();

        println!("context::capture_active_context: total_len_utf16={} before_chars={} after_chars={}", len, before.chars().count(), after.chars().count());
        crate::logger::log(&format!("context: len={} before_chars={} after_chars={}", len, before.chars().count(), after.chars().count()));

        Ok(TextContext { before, after })
    }
}

/// Convert a UTF-16 code-unit offset to a Rust String character index.
/// Handles surrogate pairs (emoji, etc.) correctly.
fn utf16_offset_to_char_index(utf16_buf: &[u16], utf16_offset: usize) -> usize {
    let mut char_index = 0;
    let mut utf16_pos = 0;

    while utf16_pos < utf16_offset && utf16_pos < utf16_buf.len() {
        let code_unit = utf16_buf[utf16_pos];
        // High surrogate (0xD800..0xDBFF): advances by 2 code units
        if (0xD800..=0xDBFF).contains(&code_unit) {
            utf16_pos += 2;
        } else {
            utf16_pos += 1;
        }
        char_index += 1;
    }

    char_index
}

/// Capture screenshot of active window for AI context
pub fn capture_screen_context() -> Result<String> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == 0 {
            return Ok("No active window".to_string());
        }
        
        // Get window title
        let mut title_buf: Vec<u16> = vec![0u16; 256];
        let title_len = GetWindowTextW(hwnd, &mut title_buf);
        let window_title = if title_len > 0 {
            String::from_utf16_lossy(&title_buf[..title_len as usize])
        } else {
            "Unknown Window".to_string()
        };
        
        // Try to capture screenshot
        let screen_desc = "Window class and title captured for context".to_string();
        
        // Get window class name
        let mut class_buf: Vec<u16> = vec![0u16; 256];
        let class_len = GetClassNameW(hwnd, &mut class_buf);
        let window_class = if class_len > 0 {
            String::from_utf16_lossy(&class_buf[..class_len as usize])
        } else {
            "Unknown".to_string()
        };
        
        Ok(format!("Active Window: {}\\nWindow Class: {}\\n{}", window_title, window_class, screen_desc))
    }
}
