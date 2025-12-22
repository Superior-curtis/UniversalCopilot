use anyhow::Result;
use reqwest::Client;
use futures_util::StreamExt;

const OLLAMA_API_URL: &str = "http://localhost:11434/api/generate";
const MODEL: &str = "mistral:latest";

/// Synchronous query for chatbot (blocking)
pub fn query_ollama(user_message: &str) -> Result<String> {
    use std::io::Read;
    
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(6))
        .build()?;
    let body = serde_json::json!({
        "model": MODEL,
        "prompt": user_message,
        "stream": false,
        "temperature": 0.4,
    });
    
    let mut resp = client
        .post(OLLAMA_API_URL)
        .json(&body)
        .send()?;
    
    let mut response_text = String::new();
    resp.read_to_string(&mut response_text)?;
    
    let json: serde_json::Value = serde_json::from_str(&response_text)?;
    Ok(json["response"].as_str().unwrap_or("").to_string())
}

/// Async streaming query for chat with progress callback
pub async fn query_ollama_streaming<F>(user_message: &str, mut on_chunk: F) -> Result<String>
where
    F: FnMut(&str),
{
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;
    
    let body = serde_json::json!({
        "model": MODEL,
        "prompt": user_message,
        "stream": true,
        "temperature": 0.5,
    });
    
    let resp = client
        .post(OLLAMA_API_URL)
        .json(&body)
        .send()
        .await?;
    
    let mut full_response = String::new();
    let mut stream = resp.bytes_stream();
    
    while let Some(chunk_result) = stream.next().await {
        if let Ok(chunk) = chunk_result {
            if let Ok(text) = String::from_utf8(chunk.to_vec()) {
                for line in text.lines() {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                        if let Some(response_chunk) = json.get("response").and_then(|v| v.as_str()) {
                            full_response.push_str(response_chunk);
                            on_chunk(response_chunk);
                        }
                    }
                }
            }
        }
    }
    
    Ok(full_response)
}

/// Initialize/check local LLM availability
use once_cell::sync::OnceCell;
static INIT_OK: OnceCell<bool> = OnceCell::new();

pub async fn init_model() -> Result<()> {
    if INIT_OK.get().copied() == Some(true) {
        return Ok(());
    }
    println!("llm: checking for Ollama at localhost:11434");
    crate::logger::log("llm: checking Ollama availability");
    
    let client = Client::new();
    match client.get("http://localhost:11434/api/tags").send().await {
        Ok(resp) if resp.status().is_success() => {
            println!("llm: Ollama is available");
            crate::logger::log("llm: Ollama available");
            let _ = INIT_OK.set(true);
            Ok(())
        }
        _ => {
            println!("llm: Ollama not found at localhost:11434");
            println!("llm: Please install Ollama from https://ollama.ai");
            crate::logger::log("llm: Ollama not available - install from https://ollama.ai");
            let _ = INIT_OK.set(false);
            Err(anyhow::anyhow!("Ollama not available. Please install from https://ollama.ai and run: ollama pull mistral"))
        }
    }
}

/// Generate inline suggestion using Ollama
pub async fn generate_suggestion(context: &str) -> Result<String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(6))
        .build()?;
    
    // Better prompt focused on continuing user's writing
    let trimmed = context.trim();
    crate::logger::log(&format!("llm: generate_suggestion called with context_len={}, trimmed_len={}", context.len(), trimmed.len()));
    
    // If context is too short, still try but with a different prompt
    if trimmed.is_empty() {
        crate::logger::log("llm: context is empty, returning empty suggestion");
        return Ok(String::new());
    }

    let tail: String = trimmed.chars().rev().take(200).collect::<String>().chars().rev().collect::<String>();
    let prompt = format!(
        "Continue the user's writing with 2–3 complete sentences.\n\
         Correct minor grammar/spelling mistakes if present, but preserve tone and meaning.\n\
         Only output the continuation (no prefaces).\n\
         Text tail to continue:\n{}",
        tail
    );
    
    println!("llm: generating suggestion for context_len={}", context.len());
    crate::logger::log(&format!("llm: generating suggestion context_len={}", context.len()));
    
    let body = serde_json::json!({
        "model": MODEL,
        "prompt": prompt,
        "stream": false,
        "temperature": 0.7,
    });
    
    match client.post(OLLAMA_API_URL).json(&body).send().await {
        Ok(resp) => {
            crate::logger::log(&format!("llm: response status={}", resp.status()));
            if !resp.status().is_success() {
                let err_msg = format!("Ollama request failed: {}", resp.status());
                crate::logger::log(&format!("llm: {}", err_msg));
                return Err(anyhow::anyhow!(err_msg));
            }
            
            match resp.json::<serde_json::Value>().await {
                Ok(response_body) => {
                    if let Some(response_text) = response_body.get("response").and_then(|v| v.as_str()) {
                        let suggestion = response_text.trim().to_string();
                        crate::logger::log(&format!("llm: suggestion_len={}", suggestion.len()));
                        if !suggestion.is_empty() {
                            crate::logger::log(&format!("llm: suggestion returned successfully"));
                            println!("llm: suggestion={}", suggestion);
                            return Ok(suggestion);
                        } else {
                            crate::logger::log("llm: suggestion is empty after trim");
                        }
                    } else {
                        crate::logger::log("llm: no response field in json");
                    }
                    Ok(String::new())
                }
                Err(e) => {
                    let err_msg = format!("Failed to parse response JSON: {}", e);
                    crate::logger::log(&format!("llm: {}", err_msg));
                    Err(anyhow::anyhow!(err_msg))
                }
            }
        }
        Err(e) => {
            let err_msg = format!("HTTP error: {}", e);
            crate::logger::log(&format!("llm: {}", err_msg));
            Err(anyhow::anyhow!(err_msg))
        }
    }
}
