use parking_lot::Mutex;
use std::mem::transmute;
use std::sync::Arc;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{BeginPaint, CreateFontW, CreateSolidBrush, DeleteObject, EndPaint, FillRect, HBRUSH, HDC, HGDIOBJ, HFONT, PAINTSTRUCT, RoundRect, SelectObject, SetBkColor, SetTextColor};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{VK_ESCAPE, VK_RETURN};
use windows::Win32::UI::WindowsAndMessaging::{CallWindowProcW, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW, GetParent, GetWindowTextLengthW, GetWindowTextW, LoadCursorW, MoveWindow, PostQuitMessage, RegisterClassW, SendMessageW, SetForegroundWindow, SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowTextW, ShowWindow, TranslateMessage, GWLP_WNDPROC, WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASSW, WM_CHAR, WM_CLOSE, WM_CREATE, WM_CTLCOLOREDIT, WM_DESTROY, WM_KEYDOWN, WM_MOUSEWHEEL, WM_PAINT, WM_SETFONT, WM_SIZE, WM_VSCROLL, WS_BORDER, WS_CAPTION, WS_CHILD, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_MINIMIZEBOX, WS_POPUP, WS_SYSMENU, WS_TABSTOP, WS_VSCROLL, WS_VISIBLE, ES_AUTOVSCROLL, ES_AUTOHSCROLL, ES_MULTILINE, ES_READONLY, SW_HIDE, SW_SHOW, IDC_ARROW, LWA_ALPHA, MSG};
use once_cell::sync::OnceCell;

static CHAT_HISTORY: OnceCell<Arc<Mutex<Vec<ChatMessage>>>> = OnceCell::new();
static HISTORY_EDIT: OnceCell<HWND> = OnceCell::new();
static INPUT_EDIT: OnceCell<HWND> = OnceCell::new();
static UI_FONT: OnceCell<HFONT> = OnceCell::new();
static EDIT_BRUSH: OnceCell<HBRUSH> = OnceCell::new();
static HISTORY_BRUSH: OnceCell<HBRUSH> = OnceCell::new();
static INPUT_BRUSH: OnceCell<HBRUSH> = OnceCell::new();
static ORIGINAL_INPUT_WNDPROC: OnceCell<isize> = OnceCell::new();
static mut CHAT_HWND: HWND = HWND(0);
static mut VISIBLE: bool = false;
// Cached non-chat context captured right before showing the chat window
static CHAT_CTX_SCREEN: OnceCell<Arc<Mutex<Option<String>>>> = OnceCell::new();
static CHAT_CTX_TEXT: OnceCell<Arc<Mutex<Option<String>>>> = OnceCell::new();
const BG_COLOR: COLORREF = COLORREF(0x00141418);
const PANEL_COLOR: COLORREF = COLORREF(0x00261E1E);
const INPUT_BG_COLOR: COLORREF = COLORREF(0x0030333A);
const EM_SETBKGNDCOLOR: u32 = 0x00D1;
const EM_SETSEL: u32 = 0x00B1;
const EM_SCROLLCARET: u32 = 0x00B7;
const EM_SETCUEBANNER: u32 = 0x1501;

#[derive(Clone)]
pub struct ChatMessage {
    pub is_user: bool,
    pub text: String,
}

// Returns the latest AI message text (skipping placeholders), if any
pub fn get_last_ai_text() -> Option<String> {
    if let Some(history) = CHAT_HISTORY.get() {
        let msgs = history.lock();
        for msg in msgs.iter().rev() {
            if !msg.is_user {
                let t = msg.text.trim();
                if !t.is_empty() && t != "Thinking..." {
                    // Prefer a short, clean payload for auto-typing
                    return Some(t.to_string());
                }
            }
        }
    }
    None
}

pub fn is_chat_visible() -> bool {
    unsafe { VISIBLE }
}

fn to_wide(s: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

fn layout_children(hwnd: HWND) {
    unsafe {
        let mut client = RECT::default();
        let _ = GetClientRect(hwnd, &mut client);
        let panel = RECT { left: 8, top: 8, right: client.right - 8, bottom: client.bottom - 8 };
        let history_height = (panel.bottom - panel.top) - 84;
        let history_left = panel.left + 12;
        let history_top = panel.top + 12;
        let history_w = (panel.right - panel.left) - 24;
        let input_h = 48;
        let input_left = panel.left + 12;
        let input_top = panel.bottom - input_h - 12;
        let input_w = history_w;

        if let Some(h) = HISTORY_EDIT.get() {
            MoveWindow(*h, history_left, history_top, history_w, history_height, true);
        }
        if let Some(h) = INPUT_EDIT.get() {
            MoveWindow(*h, input_left, input_top, input_w, input_h, true);
        }
    }
}

fn update_history_edit() {
    println!("update_history_edit: Starting capture...");
    if let (Some(history), Some(edit)) = (CHAT_HISTORY.get(), HISTORY_EDIT.get()) {
        let msgs = history.lock();
        let mut combined = String::new();
        
        // Show what AI can see from active window
        combined.push_str("=== What I Can See ===\r\n");
        // Prefer cached non-chat contexts while the chat window is visible
        let cached_text = CHAT_CTX_TEXT
            .get_or_init(|| Arc::new(Mutex::new(None)))
            .lock()
            .clone();
        let cached_screen = CHAT_CTX_SCREEN
            .get_or_init(|| Arc::new(Mutex::new(None)))
            .lock()
            .clone();

        if unsafe { VISIBLE } {
            if let Some(text) = cached_text {
                if text.trim().is_empty() {
                    combined.push_str("(No text content in active window)\r\n");
                } else {
                    let preview: String = text.chars().rev().take(500).collect::<String>().chars().rev().collect();
                    combined.push_str(&format!("{}\r\n", preview));
                }
            } else {
                combined.push_str("(No cached context available)\r\n");
            }
            if let Some(screen) = cached_screen {
                println!("update_history_edit: Using cached screen context: {}", screen);
                combined.push_str(&format!("Window: {}\r\n", screen));
            }
        } else {
            // Chat hidden: live capture is fine
            if let Ok(ctx) = crate::context::capture_active_context() {
                println!("update_history_edit: Captured context, before.len={}", ctx.before.len());
                if ctx.before.trim().is_empty() {
                    combined.push_str("(No text content in active window)\r\n");
                } else {
                    let preview: String = ctx.before.chars().rev().take(500).collect::<String>().chars().rev().collect();
                    combined.push_str(&format!("{}\r\n", preview));
                }
            } else {
                println!("update_history_edit: Failed to capture context");
                combined.push_str("(Can't read active window - try Notepad or text editor)\r\n");
            }
            if let Ok(screen) = crate::context::capture_screen_context() {
                println!("update_history_edit: Screen context: {}", screen);
                combined.push_str(&format!("Window: {}\r\n", screen));
            }
        }
        combined.push_str("======================\r\n\r\n");
        
        for msg in msgs.iter() {
            combined.push_str(if msg.is_user { "You: " } else { "AI: " });
            combined.push_str(&msg.text);
            combined.push_str("\r\n");
        }
        println!("update_history_edit: Writing to edit control, combined preview: {}", &combined[..combined.len().min(150)]);
        let wide = to_wide(&combined);
        unsafe {
            let _ = SetWindowTextW(*edit, PCWSTR(wide.as_ptr()));
            let len = combined.encode_utf16().count() as isize;
            SendMessageW(*edit, EM_SETSEL, WPARAM(len as usize), LPARAM(len));
            SendMessageW(*edit, EM_SCROLLCARET, WPARAM(0), LPARAM(0));
        }
    } else {
        println!("update_history_edit: CHAT_HISTORY or HISTORY_EDIT not initialized!");
    }
}

unsafe extern "system" fn input_wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_KEYDOWN {
        let key = wparam.0 as u16;
        if key == VK_RETURN.0 {
            let parent = GetParent(hwnd);
            if parent.0 != 0 {
                send_message(parent);
            }
            return LRESULT(0);
        }
        if key == VK_ESCAPE.0 {
            let parent = GetParent(hwnd);
            if parent.0 != 0 {
                ShowWindow(parent, SW_HIDE);
            }
            return LRESULT(0);
        }
    }

    let prev = ORIGINAL_INPUT_WNDPROC.get().copied().unwrap_or(0);
    CallWindowProcW(Some(transmute(prev)), hwnd, msg, wparam, lparam)
}

fn read_and_clear_input() -> String {
    if let Some(edit) = INPUT_EDIT.get() {
        unsafe {
            let len = GetWindowTextLengthW(*edit) as usize;
            if len == 0 {
                return String::new();
            }
            let mut buf = vec![0u16; len + 1];
            let _ = GetWindowTextW(*edit, &mut buf);
            let _ = SetWindowTextW(*edit, w!(""));
            String::from_utf16_lossy(&buf[..len])
        }
    } else {
        String::new()
    }
}

pub fn run_chatbot() -> anyhow::Result<()> {
    let _ = CHAT_HISTORY.set(Arc::new(Mutex::new(Vec::new())));

    unsafe {
        let hinstance = GetModuleHandleW(None).unwrap_or_default();
        let class_name = to_wide("UniversalCopilotChat");

        let wc = WNDCLASSW {
            lpfnWndProc: Some(chat_window_proc),
            hInstance: hinstance,
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hbrBackground: HBRUSH(0),
            ..Default::default()
        };

        let _ = RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED,
            PCWSTR(class_name.as_ptr()),
            windows::core::w!("AI Chat"),
            WS_POPUP | WS_BORDER | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_VSCROLL,
            50,
            50,
            420,
            520,
            HWND(0),
            None,
            hinstance,
            None,
        );

        if hwnd.0 == 0 {
            crate::logger::log("chatbot: CreateWindowExW failed");
            return Ok(());
        }

        CHAT_HWND = hwnd;
        crate::logger::log(&format!("chatbot: window created HWND={}", hwnd.0));

        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 220, LWA_ALPHA);

        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);
        VISIBLE = true;

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(0), 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Ok(())
    }
}

unsafe extern "system" fn chat_window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            crate::logger::log("chatbot: WM_CREATE");

            let font = UI_FONT.get_or_init(|| {
                CreateFontW(-18, 0, 0, 0, 400, 0, 0, 0, 1, 4, 0, 5, 0, PCWSTR(to_wide("Segoe UI").as_ptr()))
            });
            let _edit_brush = EDIT_BRUSH.get_or_init(|| CreateSolidBrush(PANEL_COLOR));
            let _history_brush = HISTORY_BRUSH.get_or_init(|| CreateSolidBrush(PANEL_COLOR));
            let _input_brush = INPUT_BRUSH.get_or_init(|| CreateSolidBrush(INPUT_BG_COLOR));

            let history_style = WINDOW_STYLE(
                WS_CHILD.0
                    | WS_VISIBLE.0
                    | WS_TABSTOP.0
                    | WS_VSCROLL.0
                    | ES_MULTILINE as u32
                    | ES_AUTOVSCROLL as u32
                    | ES_READONLY as u32,
            );
            let input_style = WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | WS_TABSTOP.0 | ES_AUTOHSCROLL as u32);

            let history_hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("EDIT"),
                w!(""),
                history_style,
                0,
                0,
                100,
                100,
                hwnd,
                None,
                None,
                None,
            );
            let input_hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("EDIT"),
                w!(""),
                input_style,
                0,
                0,
                100,
                32,
                hwnd,
                None,
                None,
                None,
            );

            if history_hwnd.0 != 0 {
                let _ = SendMessageW(history_hwnd, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
                let _ = SendMessageW(history_hwnd, EM_SETBKGNDCOLOR, WPARAM(0), LPARAM(PANEL_COLOR.0 as isize));
                let _ = HISTORY_EDIT.set(history_hwnd);
            }
            if input_hwnd.0 != 0 {
                let _ = SendMessageW(input_hwnd, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
                let _ = SendMessageW(input_hwnd, EM_SETBKGNDCOLOR, WPARAM(0), LPARAM(INPUT_BG_COLOR.0 as isize));
                let banner = to_wide("enter here");
                let _ = SendMessageW(input_hwnd, EM_SETCUEBANNER, WPARAM(1), LPARAM(banner.as_ptr() as isize));
                let prev = SetWindowLongPtrW(input_hwnd, GWLP_WNDPROC, input_wndproc as isize);
                let _ = ORIGINAL_INPUT_WNDPROC.set(prev);
                let _ = INPUT_EDIT.set(input_hwnd);
            }

            layout_children(hwnd);
            LRESULT(0)
        }
        WM_MOUSEWHEEL | WM_VSCROLL => {
            // native edit handles scrolling
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            if hdc.0 != 0 {
                let mut client = RECT::default();
                let _ = GetClientRect(hwnd, &mut client);

                let brush_win_bg = CreateSolidBrush(BG_COLOR);
                FillRect(hdc, &client, brush_win_bg);
                DeleteObject(HGDIOBJ(brush_win_bg.0 as isize));

                let panel_rect = RECT { left: 8, top: 8, right: client.right - 8, bottom: client.bottom - 8 };
                let brush_panel = CreateSolidBrush(PANEL_COLOR);
                let old_brush = SelectObject(hdc, HGDIOBJ(brush_panel.0 as isize));
                RoundRect(hdc, panel_rect.left, panel_rect.top, panel_rect.right, panel_rect.bottom, 16, 16);
                DeleteObject(HGDIOBJ(brush_panel.0 as isize));
                let _ = SelectObject(hdc, old_brush);
            }
            EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_CTLCOLOREDIT => {
            let hdc = HDC(wparam.0 as isize);
            let target = HWND(lparam.0);
            if let Some(input) = INPUT_EDIT.get() {
                if target.0 == input.0 {
                    SetBkColor(hdc, INPUT_BG_COLOR);
                    SetTextColor(hdc, COLORREF(0x00E0E0E0));
                    if let Some(brush) = INPUT_BRUSH.get() {
                        return LRESULT(brush.0 as isize);
                    }
                }
            }
            SetBkColor(hdc, PANEL_COLOR);
            SetTextColor(hdc, COLORREF(0x00D0D0D0));
            if let Some(brush) = HISTORY_BRUSH.get() {
                return LRESULT(brush.0 as isize);
            }
            LRESULT(0)
        }
        WM_CHAR => {
            if wparam.0 as u32 == 27 {
                ShowWindow(hwnd, SW_HIDE);
                VISIBLE = false;
            }
            LRESULT(0)
        }
        WM_SIZE => {
            layout_children(hwnd);
            LRESULT(0)
        }
        WM_CLOSE => {
            ShowWindow(hwnd, SW_HIDE);
            VISIBLE = false;
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn send_message(_hwnd: HWND) {
    let user_msg = read_and_clear_input();

    if user_msg.trim().is_empty() {
        return;
    }

    if let Some(history) = CHAT_HISTORY.get() {
        let mut msgs = history.lock();
        msgs.push(ChatMessage { is_user: true, text: user_msg.clone() });
        msgs.push(ChatMessage { is_user: false, text: "Thinking...".to_string() });
    }
    update_history_edit();

    std::thread::spawn(move || {
        // Use cached contexts while chat is visible to avoid switching to the chat window
        let screen_context = {
            let cached = CHAT_CTX_SCREEN.get_or_init(|| Arc::new(Mutex::new(None))).lock().clone();
            if unsafe { VISIBLE } {
                cached.unwrap_or_else(|| "No screen context available".to_string())
            } else {
                match crate::context::capture_screen_context() {
                    Ok(ctx) => ctx,
                    Err(_) => "No screen context available".to_string(),
                }
            }
        };
        let text_context = {
            let cached = CHAT_CTX_TEXT.get_or_init(|| Arc::new(Mutex::new(None))).lock().clone();
            if unsafe { VISIBLE } {
                cached.unwrap_or_default()
            } else {
                match crate::context::capture_active_context() {
                    Ok(c) => c.before.chars().rev().take(400).collect::<String>().chars().rev().collect::<String>(),
                    Err(_) => String::new(),
                }
            }
        };

        let enhanced_prompt = format!(
            "[System Context]\n{}\n\n[User Text Near Caret]\n{}\n\n[User Message]\n{}",
            screen_context,
            text_context,
            user_msg
        );

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut ai_response = String::new();
            let result = crate::llm::query_ollama_streaming(&enhanced_prompt, |chunk| {
                ai_response.push_str(chunk);
                if let Some(history) = CHAT_HISTORY.get() {
                    let mut msgs = history.lock();
                    if let Some(last) = msgs.last_mut() {
                        if !last.is_user {
                            last.text = ai_response.clone();
                        }
                    }
                }
                update_history_edit();
            }).await;

            if result.is_err() {
                if let Some(history) = CHAT_HISTORY.get() {
                    let mut msgs = history.lock();
                    if let Some(last) = msgs.last_mut() {
                        if !last.is_user {
                            last.text = "AI unavailable. Make sure Ollama is running (localhost:11434).".to_string();
                        }
                    }
                }
                update_history_edit();
            }
        });
    });
}

pub fn toggle_chatbot() {
    unsafe {
        if CHAT_HWND.0 != 0 {
            VISIBLE = !VISIBLE;
            if VISIBLE {
                println!("toggle_chatbot: About to capture context...");
                // Capture context BEFORE showing chat window and cache it
                let screen_context = match crate::context::capture_screen_context() {
                    Ok(ctx) => ctx,
                    Err(_) => "".to_string(),
                };
                let text_context = match crate::context::capture_active_context() {
                    Ok(c) => c.before.chars().rev().take(500).collect::<String>().chars().rev().collect::<String>(),
                    Err(_) => String::new(),
                };
                {
                    let screen_cell = CHAT_CTX_SCREEN.get_or_init(|| Arc::new(Mutex::new(None)));
                    *screen_cell.lock() = Some(screen_context);
                    let text_cell = CHAT_CTX_TEXT.get_or_init(|| Arc::new(Mutex::new(None)));
                    *text_cell.lock() = Some(text_context);
                }
                update_history_edit();
                println!("toggle_chatbot: Context captured, showing window");
            }
            ShowWindow(CHAT_HWND, if VISIBLE { SW_SHOW } else { SW_HIDE });
            if VISIBLE {
                let _ = SetForegroundWindow(CHAT_HWND);
            }
            crate::logger::log(&format!("chatbot: toggled to {}", if VISIBLE { "visible" } else { "hidden" }));
        }
    }
}
