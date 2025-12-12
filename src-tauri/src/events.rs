use tauri::Emitter;

pub fn send_event(app_handle: &tauri::AppHandle, event: &str) {
    println!("[Event] Sending event: {}", event);

    // app_handle.emit(event, payload)
}
