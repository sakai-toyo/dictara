use tauri_plugin_opener::OpenerExt;

// ===== MICROPHONE PERMISSION COMMANDS =====

/// Check microphone permission status
/// Returns: "authorized", "denied", "restricted", or "not_determined"
#[tauri::command]
#[specta::specta]
pub fn check_microphone_permission() -> String {
    #[cfg(target_os = "macos")]
    {
        use objc2_av_foundation::{AVAuthorizationStatus, AVCaptureDevice, AVMediaTypeAudio};

        // Safety: AVMediaTypeAudio is an extern static that must be accessed in unsafe block
        let media_type = match unsafe { AVMediaTypeAudio } {
            Some(mt) => mt,
            None => return "unknown".to_string(),
        };

        // Safety: media_type is a valid NSString reference
        let status = unsafe { AVCaptureDevice::authorizationStatusForMediaType(media_type) };

        match status {
            AVAuthorizationStatus::Authorized => "authorized".to_string(),
            AVAuthorizationStatus::Denied => "denied".to_string(),
            AVAuthorizationStatus::Restricted => "restricted".to_string(),
            AVAuthorizationStatus::NotDetermined => "not_determined".to_string(),
            _ => "unknown".to_string(),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        "authorized".to_string() // Other platforms don't need this permission
    }
}

/// Open System Settings to the Microphone privacy pane
#[tauri::command]
#[specta::specta]
pub fn open_microphone_settings(app: tauri::AppHandle) {
    #[cfg(target_os = "macos")]
    {
        // macOS URL scheme for Privacy & Security > Microphone
        let _ = app.opener().open_url(
            "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone",
            None::<&str>,
        );
    }
}
