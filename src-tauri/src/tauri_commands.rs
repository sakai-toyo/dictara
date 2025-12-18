use crate::keychain;
use crate::recording::RecordingCommand;
use crate::setup::{AudioLevelChannel, RecordingCommandSender};
use tauri::State;
use tauri::ipc::Channel;

#[tauri::command]
pub fn check_accessibility_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        macos_accessibility_client::accessibility::application_is_trusted()
    }
    #[cfg(not(target_os = "macos"))]
    {
        true // Other platforms don't need this permission
    }
}

#[tauri::command]
pub fn request_accessibility_permission() {
    #[cfg(target_os = "macos")]
    {
        // This will show macOS system dialog and open System Settings
        macos_accessibility_client::accessibility::application_is_trusted_with_prompt();
    }
}

#[tauri::command]
pub fn restart_app(app: tauri::AppHandle) {
    app.restart();
}

#[tauri::command]
pub fn stop_recording(sender: State<RecordingCommandSender>) -> Result<(), String> {
    sender.sender.blocking_send(RecordingCommand::FnUp)
        .map_err(|e| format!("Failed to send FnUp command: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn cancel_recording(sender: State<RecordingCommandSender>) -> Result<(), String> {
    sender.sender.blocking_send(RecordingCommand::Cancel)
        .map_err(|e| format!("Failed to send Cancel command: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn save_openai_key(key: String) -> Result<(), String> {
    println!("[Command] save_openai_key called with key length: {}", key.len());
    keychain::save_api_key(&key).map_err(|e| {
        let error = format!("Failed to save API key: {}", e);
        eprintln!("[Command] {}", error);
        error
    })
}

#[tauri::command]
pub fn load_openai_key() -> Result<Option<String>, String> {
    println!("[Command] load_openai_key called");
    keychain::load_api_key().map_err(|e| {
        let error = format!("Failed to load API key: {}", e);
        eprintln!("[Command] {}", error);
        error
    })
}

#[tauri::command]
pub fn delete_openai_key() -> Result<(), String> {
    println!("[Command] delete_openai_key called");
    keychain::delete_api_key().map_err(|e| {
        let error = format!("Failed to delete API key: {}", e);
        eprintln!("[Command] {}", error);
        error
    })
}

#[tauri::command]
pub fn test_openai_key(key: String) -> Result<bool, String> {
    println!("[Command] test_openai_key called");

    use crate::clients::openai::OpenAIClient;

    OpenAIClient::test_api_key(&key).map_err(|e| {
        let error = format!("Failed to test API key: {}", e);
        eprintln!("[Command] {}", error);
        error
    })
}

#[tauri::command]
pub fn register_audio_level_channel(
    channel: Channel<f32>,
    state: State<AudioLevelChannel>,
) -> Result<(), String> {
    let mut channel_lock = state.channel.lock().unwrap();
    *channel_lock = Some(channel);
    Ok(())
}
