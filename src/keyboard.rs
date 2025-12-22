use anyhow::Result;
use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use std::sync::Arc;
use windows::Win32::Foundation::{LRESULT, WPARAM, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, SendInput, KEYEVENTF_UNICODE, KEYEVENTF_KEYUP, VIRTUAL_KEY, KEYBD_EVENT_FLAGS, GetAsyncKeyState, VK_CONTROL, VK_SHIFT, GetKeyboardState, ToUnicode};
use windows::Win32::Foundation::HWND;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch;

static SUGGESTION_CELL: OnceCell<Arc<Mutex<String>>> = OnceCell::new();
static mut HOOK_HANDLE: HHOOK = HHOOK(0);
static NOTIFIER: OnceCell<UnboundedSender<()>> = OnceCell::new();
static CANCEL_SENDER: OnceCell<watch::Sender<bool>> = OnceCell::new();
static PATTERN_BUFFER: OnceCell<Arc<Mutex<String>>> = OnceCell::new();

fn trigger_inline_command(command: &str) {
    let cmd = command.to_string();
    println!("Inline command: {}", cmd);
    
    std::thread::spawn(move || {
        // Erase the ## pattern with backspaces
        let erase_count = 2; // ##
        unsafe {
            for _ in 0..erase_count {
                use windows::Win32::UI::Input::KeyboardAndMouse::{KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VK_BACK};
                use windows::Win32::UI::Input::KeyboardAndMouse::{INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, SendInput};
                let mut inputs = vec![
                    INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VK_BACK,
                                wScan: 0,
                                dwFlags: KEYBD_EVENT_FLAGS(0),
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    },
                    INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VK_BACK,
                                wScan: 0,
                                dwFlags: KEYEVENTF_KEYUP,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    },
                ];
                SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }

        // Capture context
        let context = match crate::context::capture_active_context() {
            Ok(c) => c.before,
            Err(_) => String::new(),
        };
        
        // Call LLM with command
        let prompt = format!(
            "User request: {}\n\nContext:\n{}\n\nProvide exactly 2 sentences to continue this text.",
            cmd,
            context
        );
        
        match crate::llm::query_ollama(&prompt) {
            Ok(response) => {
                println!("LLM response (inline overlay): {}", response);
                crate::logger::log(&format!("trigger_inline_command: LLM response received, len={}", response.len()));
                // Show as ghost text overlay; let TAB accept
                if let Some(s) = SUGGESTION_CELL.get() {
                    let mut lock = s.lock();
                    *lock = response.clone();
                    crate::logger::log(&format!("trigger_inline_command: Set suggestion text in SUGGESTION_CELL"));
                } else {
                    crate::logger::log("trigger_inline_command: WARNING - SUGGESTION_CELL not initialized!");
                }
                crate::logger::log("trigger_inline_command: Calling invalidate_overlay()");
                crate::overlay::invalidate_overlay();
            },
            Err(e) => {
                println!("LLM error: {}", e);
            }
        }
    });
}

pub fn type_suggestion(text: &str) {
    unsafe { send_text_via_sendinput(text); }
}

unsafe fn send_text_via_sendinput(text: &str) {
    // For very long payloads, use clipboard paste for reliability
    if text.chars().count() > 1500 {
        if paste_via_clipboard(text) {
            return;
        }
    }

    // Send unicode characters using SendInput with KEYEVENTF_UNICODE
    // Robust: send in chunks and handle partial sends
    let mut all_inputs: Vec<INPUT> = Vec::with_capacity(text.len() * 2);
    for ch in text.encode_utf16() {
        // Key down
        all_inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: ch,
                    dwFlags: KEYBD_EVENT_FLAGS(KEYEVENTF_UNICODE.0 as u32),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        });
        // Key up
        all_inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: ch,
                    dwFlags: KEYBD_EVENT_FLAGS((KEYEVENTF_UNICODE.0 | KEYEVENTF_KEYUP.0) as u32),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        });
    }

    // Chunked send: 64 events per batch (32 chars)
    let chunk_size: usize = 64;
    let mut offset: usize = 0;
    while offset < all_inputs.len() {
        let end = (offset + chunk_size).min(all_inputs.len());
        let batch = &all_inputs[offset..end];
        let sent = SendInput(batch, std::mem::size_of::<INPUT>() as i32);
        if sent <= 0 {
            // Brief backoff and retry once
            std::thread::sleep(std::time::Duration::from_millis(2));
            let retry = SendInput(batch, std::mem::size_of::<INPUT>() as i32);
            if retry <= 0 {
                // Bail out to avoid hanging
                break;
            }
            offset += retry as usize;
        } else {
            offset += sent as usize;
        }
        // Small yield to let target app process keystrokes
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

unsafe fn paste_via_clipboard(text: &str) -> bool {
    use windows::Win32::System::DataExchange::{OpenClipboard, CloseClipboard, EmptyClipboard, SetClipboardData};
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    use windows::Win32::Foundation::{HANDLE, HGLOBAL};
    const CF_UNICODETEXT: u32 = 13;

    // Prepare UTF-16 buffer with null terminator
    let mut wide: Vec<u16> = text.encode_utf16().collect();
    wide.push(0);

    unsafe {
        if OpenClipboard(HWND(0)).as_bool() {
            let _ = EmptyClipboard();
            let bytes = wide.len() * std::mem::size_of::<u16>();
            let hmem: HGLOBAL = match GlobalAlloc(GMEM_MOVEABLE, bytes) { Ok(h) => h, Err(_) => return false };
            if hmem.0 != 0 {
                let ptr = GlobalLock(hmem);
                if !ptr.is_null() {
                    std::ptr::copy_nonoverlapping(wide.as_ptr() as *const u8, ptr as *mut u8, bytes);
                    let _ = GlobalUnlock(hmem);
                    let _ = SetClipboardData(CF_UNICODETEXT, HANDLE(hmem.0));
                    let _ = CloseClipboard();

                    // Send Ctrl+V
                    let mut inputs: Vec<INPUT> = Vec::with_capacity(4);
                    // Ctrl down
                    inputs.push(INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT { wVk: VK_CONTROL, wScan: 0, dwFlags: KEYBD_EVENT_FLAGS(0), time: 0, dwExtraInfo: 0 },
                        },
                    });
                    // V down
                    inputs.push(INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT { wVk: VIRTUAL_KEY(0x56), wScan: 0, dwFlags: KEYBD_EVENT_FLAGS(0), time: 0, dwExtraInfo: 0 },
                        },
                    });
                    // V up
                    inputs.push(INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT { wVk: VIRTUAL_KEY(0x56), wScan: 0, dwFlags: KEYBD_EVENT_FLAGS(KEYEVENTF_KEYUP.0 as u32), time: 0, dwExtraInfo: 0 },
                        },
                    });
                    // Ctrl up
                    inputs.push(INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT { wVk: VK_CONTROL, wScan: 0, dwFlags: KEYBD_EVENT_FLAGS(KEYEVENTF_KEYUP.0 as u32), time: 0, dwExtraInfo: 0 },
                        },
                    });
                    let _ = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
                    return true;
                }
            }
            let _ = CloseClipboard();
        }
    }
    false
}

unsafe extern "system" fn ll_keyboard_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code >= 0 {
        let wm = w_param.0 as u32;
        if wm == WM_KEYDOWN || wm == WM_SYSKEYDOWN {
            let kb = *(l_param.0 as *const KBDLLHOOKSTRUCT);
            let vk = kb.vkCode as i32;
            
            // Check for Ctrl+Shift+C (toggle chatbot)
            if vk == 0x43 { // 'C' key
                let ctrl_down = unsafe { GetAsyncKeyState(VK_CONTROL.0 as i32) < 0 };
                let shift_down = unsafe { GetAsyncKeyState(VK_SHIFT.0 as i32) < 0 };
                if ctrl_down && shift_down {
                    crate::logger::log("keyboard: Ctrl+Shift+C pressed - toggle chatbot");
                    crate::chatbot::toggle_chatbot();
                    return LRESULT(1);
                }
            }

            // Ctrl+Shift+S -> summarize the active window into overlay
            if vk == 0x53 { // 'S' key
                let ctrl_down = unsafe { GetAsyncKeyState(VK_CONTROL.0 as i32) < 0 };
                let shift_down = unsafe { GetAsyncKeyState(VK_SHIFT.0 as i32) < 0 };
                if ctrl_down && shift_down {
                    crate::logger::log("keyboard: Ctrl+Shift+S pressed - summarize active window");
                    // Cancel any ongoing suggestions
                    if let Some(tx) = CANCEL_SENDER.get() { let _ = tx.send(true); }
                    // Capture context
                    let ctx = match crate::context::capture_active_context() { Ok(c) => c.before, Err(_) => String::new() };
                    let screen = match crate::context::capture_screen_context() { Ok(s) => s, Err(_) => String::new() };
                    // Prompt for concise summary
                    let prompt = format!(
                        "Summarize the following content concisely (3 bullets max or 1 short paragraph).\n\n[Window]\n{}\n\n[Text]\n{}",
                        screen,
                        ctx
                    );
                    // Query LLM synchronously
                    match crate::llm::query_ollama(&prompt) {
                        Ok(summary) => {
                            if let Some(s) = SUGGESTION_CELL.get() { let mut lock = s.lock(); *lock = summary; }
                            crate::overlay::invalidate_overlay();
                        }
                        Err(e) => {
                            if let Some(s) = SUGGESTION_CELL.get() { let mut lock = s.lock(); *lock = format!("(LLM unavailable: {})", e); }
                            crate::overlay::invalidate_overlay();
                        }
                    }
                    return LRESULT(1);
                }
            }
            
            // Removed Ctrl+Shift+A hotkey - using ## pattern instead
            
            // VK_TAB = 0x09, VK_ESCAPE = 0x1B
            if vk == 0x09 {
                // TAB -> accept suggestion (fast path)
                    println!("ll_keyboard_proc: TAB pressed");
                    crate::logger::log("keyboard: TAB pressed");
                if let Some(s) = SUGGESTION_CELL.get() {
                    let mut lock = s.lock();
                    if !lock.is_empty() {
                        let text = lock.clone();
                        // Clear and signal cancel before injection for consistency
                        *lock = String::new();
                        // Signal cancellation to async loop
                        if let Some(tx) = CANCEL_SENDER.get() {
                            let _ = tx.send(true);
                        }
                        crate::overlay::clear_suggestion();
                        // Inject text synchronously (no async overhead)
                        send_text_via_sendinput(&text);
                        // Suppress original TAB key
                        return LRESULT(1);
                    }
                }
            } else if vk == 0x1B {
                // ESC -> dismiss suggestion (fast path)
                    println!("ll_keyboard_proc: ESC pressed");
                    crate::logger::log("keyboard: ESC pressed");
                if let Some(s) = SUGGESTION_CELL.get() {
                    let mut lock = s.lock();
                    *lock = String::new();
                }
                // Signal cancellation to async loop
                if let Some(tx) = CANCEL_SENDER.get() {
                    let _ = tx.send(true);
                }
                crate::overlay::clear_suggestion();
                // Suppress original ESC key
                return LRESULT(1);
            } else {
                // Track characters for ## pattern detection
                // Convert virtual key to actual character using ToUnicode
                if let Some(buf) = PATTERN_BUFFER.get() {
                    unsafe {
                        let mut keyboard_state = [0u8; 256];
                        let _ = GetKeyboardState(&mut keyboard_state);
                        let mut result = [0u16; 5];
                        let scan_code = kb.scanCode;
                        let ret = ToUnicode(
                            vk as u32,
                            scan_code,
                            Some(&keyboard_state),
                            &mut result,
                            0,
                        );
                        
                        if ret > 0 {
                            // Got a valid character
                            if let Ok(s) = String::from_utf16(&result[0..ret as usize]) {
                                if let Some(ch) = s.chars().next() {
                                    let mut buffer = buf.lock();
                                    buffer.push(ch);
                                    println!("Tracked char: '{}' buffer len={}", ch, buffer.len());
                                    
                                    // Keep only last 200 chars
                                    if buffer.len() > 200 {
                                        buffer.drain(0..100);
                                    }
                                    
                                    // Check for ## pattern (trigger immediately on second #)
                                    if ch == '#' && buffer.ends_with("##") {
                                        println!("✓ Pattern detected! Triggering on ##");
                                        buffer.clear();
                                        // Command is empty since user just typed ##
                                        trigger_inline_command("");
                                        return LRESULT(1);
                                    }
                                }
                            }
                        }
                    }
                }
                // Other key pressed: notify inference loop to refresh prediction
                if let Some(tx) = NOTIFIER.get() {
                    let _ = tx.send(());
                }
            }
        }
    }
    // Pass to next hook (critical: never break the hook chain)
    CallNextHookEx(HHOOK(0), n_code, w_param, l_param)
}

pub fn run_keyboard_hook(suggestion: Arc<Mutex<String>>) -> Result<()> {
    unsafe {
        let _ = SUGGESTION_CELL.set(suggestion);
        let _ = PATTERN_BUFFER.set(Arc::new(Mutex::new(String::new())));

        // Use current module handle to reliably obtain HINSTANCE
        let hinstance = GetModuleHandleW(None).unwrap_or_default();
        match SetWindowsHookExW(WH_KEYBOARD_LL, Some(ll_keyboard_proc), hinstance, 0) {
            Ok(h) => {
                HOOK_HANDLE = h;
                println!("keyboard hook installed: HHOOK={} HINSTANCE={}", HOOK_HANDLE.0, hinstance.0);
            }
            Err(e) => {
                eprintln!("failed to install keyboard hook: {:?}", e);
            }
        }

        // Message loop to keep the hook alive
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(0), 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Unhook
        if HOOK_HANDLE.0 != 0 {
            UnhookWindowsHookEx(HOOK_HANDLE);
        }
    }

    Ok(())
}

pub fn set_notifier(tx: UnboundedSender<()>) {
    let _ = NOTIFIER.set(tx);
}

pub fn set_cancel_sender(tx: watch::Sender<bool>) {
    let _ = CANCEL_SENDER.set(tx);
}
