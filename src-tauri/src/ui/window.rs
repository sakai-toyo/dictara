use tauri::Manager;

const POPUP_WIDTH: u32 = 200;
const POPUP_HEIGHT: u32 = 60;
const BOTTOM_MARGIN: i32 = 200;

pub fn open_recording_popup(
    app_handle: &tauri::AppHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app_handle.get_webview_window("recording-popup") {
        // Set size
        if let Err(e) = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: POPUP_WIDTH,
            height: POPUP_HEIGHT,
        })) {
            eprintln!("[Window] Failed to set window size: {}", e);
        }

        // Get current monitor to calculate position
        if let Ok(Some(monitor)) = window.current_monitor() {
            let monitor_size = monitor.size();
            let monitor_position = monitor.position();

            // Calculate centered horizontal position
            let x = monitor_position.x + (monitor_size.width as i32 - POPUP_WIDTH as i32) / 2;

            // Calculate position 100px from bottom
            let y = monitor_position.y + monitor_size.height as i32
                - POPUP_HEIGHT as i32
                - BOTTOM_MARGIN;

            if let Err(e) =
                window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
            {
                eprintln!("[Window] Failed to set window position: {}", e);
            }
        } else {
            eprintln!("[Window] Failed to get current monitor");
        }

        if let Err(e) = window.show() {
            eprintln!("[Window] Failed to show recording popup: {}", e);
            return Err(Box::new(e));
        }
    } else {
        return Err("Recording popup window not found".into());
    }

    Ok(())
}

pub fn close_recording_popup(
    app_handle: &tauri::AppHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app_handle.get_webview_window("recording-popup") {
        if let Err(e) = window.hide() {
            eprintln!("[Window] Failed to hide recording popup: {}", e);
            return Err(Box::new(e));
        }
    } else {
        return Err("Recording popup window not found".into());
    }

    Ok(())
}
