use anyhow::Result;
use std::sync::Arc;
use parking_lot::Mutex;
use tokio::sync::watch;

// Configuration
pub const MAX_SUGGESTION_CHARS: usize = 360;
pub const MAX_TOKENS: usize = 120;
pub const MAX_IDLE_MS_BETWEEN_TOKENS: u64 = 600;

#[derive(Debug)]
pub struct Prediction {
    pub text: String,
    pub confidence: f32,
}

pub async fn predict(_context: &str) -> Result<Prediction> {
    Ok(Prediction { text: String::new(), confidence: 0.0 })
}

/// Streaming prediction using local LLM
pub async fn stream_predict(context: &str, suggestion: Arc<Mutex<String>>, _cancel_rx: watch::Receiver<bool>) -> Result<()> {
    println!("infer::stream_predict: using local LLM, context_len={}", context.len());
    crate::logger::log(&format!("infer: stream_predict context_len={}", context.len()));

    // Skip if context is completely empty
    if context.trim().is_empty() {
        crate::logger::log("infer: context is empty, skipping prediction");
        return Ok(());
    }

    // DON'T show placeholder - just wait silently for real response
    // {
    //     let mut s = suggestion.lock();
    //     if s.is_empty() {
    //         *s = "…".to_string();
    //         crate::logger::log("infer: showing placeholder");
    //     }
    // }
    // crate::overlay::invalidate_overlay();

    // Generate suggestion
    match crate::llm::generate_suggestion(context).await {
        Ok(suggestion_text) => {
            if !suggestion_text.is_empty() {
                crate::logger::log(&format!("infer: BEFORE LOCK suggestion_text_len={}", suggestion_text.len()));
                {
                    let mut s = suggestion.lock();
                    *s = suggestion_text.clone();
                    crate::logger::log(&format!("infer: SET suggestion, now={}", s.len()));
                }
                crate::logger::log(&format!("infer: AFTER LOCK, calling invalidate"));
                crate::logger::log(&format!("infer: appended_token text_len={}", suggestion_text.len()));
                crate::overlay::invalidate_overlay();
                println!("infer: suggestion={}", suggestion_text);
            } else {
                crate::logger::log("infer: suggestion_text is empty");
                // Keep the previous suggestion, don't clear it
            }
        }
        Err(e) => {
            println!("infer: generation failed: {}", e);
            crate::logger::log(&format!("infer: generation_failed: {}", e));
            // Keep previous suggestion on error, don't clear
        }
    }

    Ok(())
}
