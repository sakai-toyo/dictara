use std::time::Duration;
use std::time::Instant;

use log::{error, info, warn};
use serde_json::{json, Value};

use crate::config::OpenAIConfig;
use crate::keychain::{self, ProviderAccount};

const OPENAI_RESPONSES_URL: &str = "https://api.openai.com/v1/responses";
const OPENAI_POST_PROCESS_TIMEOUT_SECS: u64 = 10;

/// Best-effort post-processing with OpenAI Responses API.
///
/// If OpenAI key/config is missing or request/parsing fails, returns original text unchanged.
pub fn post_process_with_openai(text: &str, model: &str, prompt: &str) -> String {
    let started_at = Instant::now();
    let trimmed = text.trim();
    let trimmed_model = model.trim();
    let trimmed_prompt = prompt.trim();

    if trimmed.is_empty() {
        info!("Post-processing skipped: empty transcription");
        return text.to_string();
    }
    if trimmed_model.is_empty() {
        warn!("Post-processing skipped: model is empty");
        return text.to_string();
    }
    if trimmed_prompt.is_empty() {
        warn!("Post-processing skipped: prompt is empty");
        return text.to_string();
    }

    let openai_config =
        match keychain::load_provider_config::<OpenAIConfig>(ProviderAccount::OpenAI) {
            Ok(Some(config)) => config,
            Ok(None) => {
                warn!("OpenAI config not found, skipping post-processing");
                return text.to_string();
            }
            Err(e) => {
                error!("Failed to load OpenAI config from keychain: {e}");
                return text.to_string();
            }
        };

    let http_client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(OPENAI_POST_PROCESS_TIMEOUT_SECS))
        .build()
    {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to create HTTP client for post-processing: {e}");
            return text.to_string();
        }
    };

    let payload = json!({
        "model": trimmed_model,
        "instructions": trimmed_prompt,
        "input": trimmed
    });

    let response = match http_client
        .post(OPENAI_RESPONSES_URL)
        .bearer_auth(openai_config.api_key)
        .json(&payload)
        .send()
    {
        Ok(response) => response,
        Err(e) => {
            error!("Post-processing request failed: {e}");
            return text.to_string();
        }
    };

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .unwrap_or_else(|_| "Unknown error body".to_string());
        error!("Post-processing API error ({status}): {body}");
        return text.to_string();
    }

    let json: Value = match response.json() {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to parse post-processing response JSON: {e}");
            return text.to_string();
        }
    };

    if let Some(output_text) = extract_output_text(&json) {
        info!(
            "Post-processing succeeded in {}ms (input_len={}, output_len={}, changed={})",
            started_at.elapsed().as_millis(),
            trimmed.len(),
            output_text.len(),
            output_text != trimmed
        );
        return output_text;
    }

    warn!(
        "Post-processing response had no output text after {}ms, using original transcription",
        started_at.elapsed().as_millis()
    );
    text.to_string()
}

fn extract_output_text(response_json: &Value) -> Option<String> {
    // Some API shapes include top-level "output_text"
    if let Some(text) = response_json.get("output_text").and_then(Value::as_str) {
        let text = text.trim();
        if !text.is_empty() {
            return Some(text.to_string());
        }
    }

    // General responses shape:
    // output[].content[] where content.type == "output_text" and content.text is the actual text.
    let output = response_json.get("output")?.as_array()?;
    let mut merged = String::new();

    for item in output {
        let content = match item.get("content").and_then(Value::as_array) {
            Some(content) => content,
            None => continue,
        };

        for content_item in content {
            let item_type = content_item.get("type").and_then(Value::as_str);
            if item_type != Some("output_text") {
                continue;
            }

            let text = match content_item.get("text").and_then(Value::as_str) {
                Some(text) => text.trim(),
                None => continue,
            };

            if text.is_empty() {
                continue;
            }

            if !merged.is_empty() {
                merged.push('\n');
            }
            merged.push_str(text);
        }
    }

    if merged.is_empty() {
        None
    } else {
        Some(merged)
    }
}
