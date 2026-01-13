use crate::config::{self, Provider};
use log::error;
use tauri_plugin_store::StoreExt;

// ===== PROVIDER SELECTION COMMANDS =====

/// Get the currently active provider
#[tauri::command]
#[specta::specta]
pub fn get_current_provider(app: tauri::AppHandle) -> Result<Option<Provider>, String> {
    let store = app.store("config.json").map_err(|e| {
        error!("Failed to open store: {}", e);
        format!("Failed to open store: {}", e)
    })?;

    let config = config::load_app_config(&store);
    Ok(config.active_provider)
}

/// Set the currently active provider
#[tauri::command]
#[specta::specta]
pub fn set_current_provider(app: tauri::AppHandle, provider: String) -> Result<(), String> {
    let store = app.store("config.json").map_err(|e| {
        error!("Failed to open store: {}", e);
        format!("Failed to open store: {}", e)
    })?;

    // Load existing config to preserve other fields
    let mut config = config::load_app_config(&store);

    // Parse and set the provider
    config.active_provider = Some(match provider.as_str() {
        "open_ai" | "openai" => Provider::OpenAI,
        "azure_open_ai" | "azure_openai" | "azure" => Provider::AzureOpenAI,
        "local" => Provider::Local,
        _ => {
            error!("Invalid provider: {}", provider);
            return Err(format!("Invalid provider: {}", provider));
        }
    });

    config::save_app_config(&store, &config)
}

/// Clear the currently active provider (set to None)
#[tauri::command]
#[specta::specta]
pub fn clear_current_provider(app: tauri::AppHandle) -> Result<(), String> {
    let store = app.store("config.json").map_err(|e| {
        error!("Failed to open store: {}", e);
        format!("Failed to open store: {}", e)
    })?;

    // Load existing config to preserve other fields
    let mut config = config::load_app_config(&store);
    config.active_provider = None;

    config::save_app_config(&store, &config)
}
