use crate::config::{self, AppConfig, Provider, RecordingTrigger};
use log::error;
use tauri_plugin_store::StoreExt;

// ===== GENERAL APP CONFIGURATION COMMANDS =====

/// Load the entire app configuration
#[tauri::command]
#[specta::specta]
pub fn load_app_config(app: tauri::AppHandle) -> Result<AppConfig, String> {
    let store = app.store("config.json").map_err(|e| {
        error!("Failed to open store: {}", e);
        format!("Failed to open store: {}", e)
    })?;

    Ok(config::load_app_config(&store))
}

/// Save app configuration (general-purpose command that can update multiple fields)
#[tauri::command]
#[specta::specta]
pub fn save_app_config(
    app: tauri::AppHandle,
    active_provider: Option<String>,
    recording_trigger: Option<RecordingTrigger>,
) -> Result<(), String> {
    let store = app.store("config.json").map_err(|e| {
        error!("Failed to open store: {}", e);
        format!("Failed to open store: {}", e)
    })?;

    // Load existing config to preserve fields that aren't being updated
    let mut config = config::load_app_config(&store);

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

    config::save_app_config(&store, &config)
}
