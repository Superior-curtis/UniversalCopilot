use anyhow::Result;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

mod overlay;
mod caret;
mod keyboard;
mod infer;
mod context;
mod logger;
mod llm;
mod chatbot;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Universal Copilot Phase A (Windows) - prototype starting");
    logger::log("main: starting");

    let suggestion = Arc::new(Mutex::new(String::new()));

    // start overlay window (renders ghost text from `suggestion`)
    logger::log("main: about to spawn overlay thread");
    let ov_sugg = suggestion.clone();
    let overlay_handle = std::thread::spawn(move || {
        logger::log("main: overlay thread spawned, calling run_overlay()");
        overlay::run_overlay(ov_sugg).expect("overlay failed");
    });

    // start chatbot window (toggle with Ctrl+Shift+C)
    logger::log("main: about to spawn chatbot thread");
    let _chatbot_handle = std::thread::spawn(move || {
        logger::log("main: chatbot thread spawned, calling run_chatbot()");
        chatbot::run_chatbot().expect("chatbot failed");
    });

    // start caret poller
    let _ctl_sugg = suggestion.clone();
    let caret_handle = tokio::spawn(async move {
        loop {
            // Always update position for responsiveness (uses caret or mouse fallback)
            match caret::get_caret_rect() {
                Ok(rect) => {
                    overlay::update_overlay_position(rect.x, rect.y);
                }
                Err(_) => {
                    // Silent failure
                }
            }
            // Faster polling for smoother tracking
            sleep(Duration::from_millis(60)).await;
        }
    });

    // create notifier channel for immediate prediction requests
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();

    // set notifier for keyboard hook
    keyboard::set_notifier(tx);

    // start keyboard hook (TAB/ESC handling)
    let kb_sugg = suggestion.clone();
    let kb_handle = std::thread::spawn(move || {
        keyboard::run_keyboard_hook(kb_sugg).expect("keyboard hook failed");
    });

    // inference client loop with streaming support (cancellable)
    let inf_sugg = suggestion.clone();
    let mut active_cancel: Option<tokio::sync::watch::Sender<bool>> = None;
    let mut last_run = std::time::Instant::now();
    let inf_handle = tokio::spawn(async move {
        loop {
            // Wait for keyboard input OR periodic timer for continuous help
            tokio::select! {
                biased;
                _ = rx.recv() => {
                    // User typing - respond immediately after debounce
                }
                _ = sleep(Duration::from_millis(1200)) => {
                    // Periodic check even with no typing
                }
            }

            // Short debounce: 400ms of no typing before generating (user still typing = don't interrupt)
            tokio::select! {
                _ = sleep(Duration::from_millis(400)) => {}
                _ = rx.recv() => {
                    // More typing arrived; restart debounce loop
                    continue;
                }
            }

            // cancel any previous streaming
            if let Some(tx) = active_cancel.take() {
                let _ = tx.send(true);
            }

            // capture active control context (text before caret)
            let ctx = match context::capture_active_context() {
                Ok(c) => c.before,
                Err(_) => {
                    // Silent failure: if context capture fails, use empty context
                    String::new()
                }
            };

            // create a new cancel channel for the new stream
            let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
            // expose cancel sender to keyboard module so TAB/ESC can cancel streams
            keyboard::set_cancel_sender(cancel_tx.clone());
            active_cancel = Some(cancel_tx);

            // Don't clear current suggestion - let it persist until a new one arrives

            // spawn the streaming prediction task and await it (so we keep only one active)
            let s_clone = inf_sugg.clone();
            let cancel_rx_local = cancel_rx.clone();
            let ctx_clone = ctx.clone();
            let stream_task = tokio::spawn(async move {
                // Silent failure: inference errors don't crash the loop or spam overlay
                let _ = infer::stream_predict(&ctx_clone, s_clone, cancel_rx_local).await;
            });

            // wait for the stream to finish
            let _ = stream_task.await;
            last_run = std::time::Instant::now();
        }
    });

    // keep main alive
    caret_handle.await?;
    overlay_handle.join().ok();
    kb_handle.join().ok();
    inf_handle.await?;

    Ok(())
}
