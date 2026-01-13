use crate::config::{self, OnboardingConfig, OnboardingStep};
use crate::ui::window;
use log::error;
use tauri_plugin_store::StoreExt;

// ===== ONBOARDING FLOW COMMANDS =====

#[tauri::command]
pub fn restart_app(app: tauri::AppHandle) {
    app.restart();
}

#[tauri::command]
#[specta::specta]
pub fn load_onboarding_config(app: tauri::AppHandle) -> Result<OnboardingConfig, String> {
    let store = app.store("config.json").map_err(|e| {
        error!("Failed to open store: {}", e);
        format!("Failed to open store: {}", e)
    })?;

    Ok(config::load_onboarding_config(&store))
}

#[tauri::command]
#[specta::specta]
pub fn save_onboarding_step(app: tauri::AppHandle, step: OnboardingStep) -> Result<(), String> {
    let store = app.store("config.json").map_err(|e| {
        error!("Failed to open store: {}", e);
        format!("Failed to open store: {}", e)
    })?;

    let mut onboarding_config = config::load_onboarding_config(&store);
    onboarding_config.current_step = step;
    config::save_onboarding_config(&store, &onboarding_config)
}

#[tauri::command]
#[specta::specta]
pub fn finish_onboarding(app: tauri::AppHandle) -> Result<(), String> {
    let store = app.store("config.json").map_err(|e| {
        error!("Failed to open store: {}", e);
        format!("Failed to open store: {}", e)
    })?;

    let mut onboarding_config = config::load_onboarding_config(&store);
    onboarding_config.finished = true;
    onboarding_config.current_step = OnboardingStep::Complete;
    onboarding_config.pending_restart = false;
    config::save_onboarding_config(&store, &onboarding_config)?;

    // Close the onboarding window
    window::close_onboarding_window(&app).map_err(|e| {
        error!("Failed to close onboarding window: {}", e);
        format!("Failed to close onboarding window: {}", e)
    })
}

#[tauri::command]
#[specta::specta]
pub fn skip_onboarding(app: tauri::AppHandle) -> Result<(), String> {
    let store = app.store("config.json").map_err(|e| {
        error!("Failed to open store: {}", e);
        format!("Failed to open store: {}", e)
    })?;

    let mut onboarding_config = config::load_onboarding_config(&store);
    onboarding_config.finished = true;
    config::save_onboarding_config(&store, &onboarding_config)?;

    // Close the onboarding window
    window::close_onboarding_window(&app).map_err(|e| {
        error!("Failed to close onboarding window: {}", e);
        format!("Failed to close onboarding window: {}", e)
    })
}

#[tauri::command]
#[specta::specta]
pub fn set_pending_restart(app: tauri::AppHandle, pending: bool) -> Result<(), String> {
    let store = app.store("config.json").map_err(|e| {
        error!("Failed to open store: {}", e);
        format!("Failed to open store: {}", e)
    })?;

    let mut onboarding_config = config::load_onboarding_config(&store);
    onboarding_config.pending_restart = pending;
    config::save_onboarding_config(&store, &onboarding_config)
}

#[tauri::command]
#[specta::specta]
pub fn restart_onboarding(app: tauri::AppHandle) -> Result<(), String> {
    let store = app.store("config.json").map_err(|e| {
        error!("Failed to open store: {}", e);
        format!("Failed to open store: {}", e)
    })?;

    // Reset onboarding config to initial state
    let onboarding_config = config::OnboardingConfig::default();
    config::save_onboarding_config(&store, &onboarding_config)?;

    // Open the onboarding window
    crate::ui::window::open_onboarding_window(&app).map_err(|e| {
        error!("Failed to open onboarding window: {}", e);
        format!("Failed to open onboarding window: {}", e)
    })
}
