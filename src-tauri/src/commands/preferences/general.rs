use crate::config::{
    self, AppConfig, ConfigKey, ConfigStore, Provider, RecordingTrigger,
    MAX_ALLOWED_SPEECH_DURATION_MS, MIN_ALLOWED_SPEECH_DURATION_MS,
};
use log::error;
use tauri::State;

// ===== GENERAL APP CONFIGURATION COMMANDS =====

/// Load the entire app configuration
#[tauri::command]
#[specta::specta]
pub fn load_app_config(config_store: State<config::Config>) -> Result<AppConfig, String> {
    Ok(config_store.get(&ConfigKey::APP).unwrap_or_default())
}

/// Save app configuration (general-purpose command that can update multiple fields)
#[tauri::command]
#[specta::specta]
pub fn save_app_config(
    config_store: State<config::Config>,
    active_provider: Option<String>,
    recording_trigger: Option<RecordingTrigger>,
    post_process_enabled: Option<bool>,
    post_process_model: Option<String>,
    post_process_prompt: Option<String>,
    min_speech_duration_ms: Option<u64>,
) -> Result<(), String> {
    // Load existing config to preserve fields that aren't being updated
    let mut config = config_store.get(&ConfigKey::APP).unwrap_or_default();

    // Update provider if specified
    if let Some(p) = active_provider {
        config.active_provider = Some(match p.as_str() {
            "open_ai" | "openai" => Provider::OpenAI,
            "azure_open_ai" | "azure_openai" | "azure" => Provider::AzureOpenAI,
            "local" => Provider::Local,
            _ => {
                error!("Invalid provider: {}", p);
                return Err(format!("Invalid provider: {}", p));
            }
        });
    }

    // Update recording trigger if specified
    if let Some(trigger) = recording_trigger {
        config.recording_trigger = trigger;
    }

    // Update post-processing enabled if specified
    if let Some(enabled) = post_process_enabled {
        config.post_process_enabled = enabled;
    }

    // Update post-processing model if specified
    if let Some(model) = post_process_model {
        let model = model.trim();
        if model.is_empty() {
            return Err("Post-process model cannot be empty".to_string());
        }
        config.post_process_model = model.to_string();
    }

    // Update post-processing prompt if specified
    if let Some(prompt) = post_process_prompt {
        let prompt = prompt.trim();
        if prompt.is_empty() {
            return Err("Post-process prompt cannot be empty".to_string());
        }
        config.post_process_prompt = prompt.to_string();
    }

    // Update minimum speech duration if specified
    if let Some(duration_ms) = min_speech_duration_ms {
        if !(MIN_ALLOWED_SPEECH_DURATION_MS..=MAX_ALLOWED_SPEECH_DURATION_MS).contains(&duration_ms)
        {
            return Err(format!(
                "min_speech_duration_ms must be between {} and {}",
                MIN_ALLOWED_SPEECH_DURATION_MS, MAX_ALLOWED_SPEECH_DURATION_MS
            ));
        }
        config.min_speech_duration_ms = duration_ms;
    }

    config_store.set(&ConfigKey::APP, config)
}
