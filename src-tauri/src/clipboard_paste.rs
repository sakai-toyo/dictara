use arboard::Clipboard;
use std::{process::Command, thread, time::Duration};

#[cfg(target_os = "macos")]
use objc2_core_graphics::{
    CGEvent, CGEventFlags, CGEventSource, CGEventSourceStateID, CGEventTapLocation, CGKeyCode,
};

/// Copy text to clipboard (without auto-pasting)
///
/// This function simply sets the text to clipboard so the user can paste it manually.
/// No keyboard simulation, no extra permissions required.
#[allow(dead_code)]
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    // Guard: Don't copy empty text
    if text.is_empty() {
        return Err("Cannot copy empty text".to_string());
    }

    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("Failed to access clipboard: {}", e))?;

    clipboard
        .set_text(text.to_string())
        .map_err(|e| format!("Failed to set clipboard text: {}", e))?;

    Ok(())
}

/// Automatically paste text at the current cursor position
///
/// This function:
/// 1. Saves the current clipboard content
/// 2. Sets the transcribed text to clipboard
/// 3. Simulates Cmd+V (macOS) or Ctrl+V (other platforms)
/// 4. Restores the original clipboard after a delay
///
/// Returns Ok(()) on success, Err on clipboard or keyboard simulation failure
///
/// WARNING: This function requires extra System Events permissions on macOS
#[allow(dead_code)]
pub fn auto_paste_text(text: &str) -> Result<(), String> {
    // Guard: Don't paste empty text
    if text.is_empty() {
        return Err("Cannot paste empty text".to_string());
    }

    // Step 1: Get clipboard instance
    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("Failed to access clipboard: {}", e))?;

    // Step 2: Save current clipboard content (if any)
    let previous_clipboard = clipboard.get_text().ok(); // Ok to fail if clipboard is empty

    // Step 3: Set transcribed text to clipboard
    clipboard
        .set_text(text.to_string())
        .map_err(|e| format!("Failed to set clipboard text: {}", e))?;

    // Step 4: Simulate paste using platform-specific methods
    // Note: Not using rdev::simulate to avoid corrupting rdev's global keyboard state
    // Note: Not using enigo because it crashes in Tauri apps on macOS (issue #6421)

    #[cfg(target_os = "macos")]
    {
        // Use AppleScript to simulate Cmd+V on macOS
        // This is more reliable than enigo in Tauri apps
        let script = r#"
            tell application "System Events"
                keystroke "v" using command down
            end tell
        "#;

        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| format!("Failed to execute AppleScript: {}", e))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("AppleScript failed: {}", error));
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        // On Windows/Linux, we'd need a different approach
        // For now, return an error - this can be implemented later
        return Err("Auto-paste not yet implemented for this platform".to_string());
    }

    // Step 5: Restore previous clipboard content after a delay
    // Spawn a background thread to restore clipboard
    if let Some(previous_text) = previous_clipboard {
        let text_for_check = text.to_string();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(150)); // Wait for paste to complete

            // Best-effort restore with race condition protection
            if let Ok(mut clipboard) = Clipboard::new() {
                // Only restore if clipboard still contains our text (avoid overwriting user's new copy)
                if let Ok(current_text) = clipboard.get_text() {
                    if current_text == text_for_check {
                        clipboard.set_text(previous_text).ok();
                    }
                }
            }
        });
    }

    Ok(())
}

/// Paste using Core Graphics events directly (Option 3)
///
/// This function creates and posts CGEvents directly to simulate Cmd+V,
/// without going through rdev::simulate() which corrupts the global LAST_FLAGS state.
///
/// HYPOTHESIS: By bypassing rdev's simulate() function and using CGEvent directly,
/// we avoid updating rdev's global LAST_FLAGS mutex, preventing FN key state corruption.
///
/// Returns Ok(()) on success, Err on event creation/posting failure
#[cfg(target_os = "macos")]
pub fn paste_with_cgevent() -> Result<(), String> {
    // Key code for 'V' key on macOS keyboard
    const V_KEYCODE: CGKeyCode = 9;

    println!("[Auto-Paste] Using Core Graphics to simulate Cmd+V");

    // Create event source for HID system state
    let event_source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .ok_or("Failed to create event source")?;

    // Step 1: Create V key press event with Command modifier
    let key_down_event = CGEvent::new_keyboard_event(
        Some(&event_source),
        V_KEYCODE,
        true  // key down
    ).ok_or("Failed to create key down event")?;

    // Set Command modifier flag (equivalent to Cmd key being held)
    CGEvent::set_flags(Some(&key_down_event), CGEventFlags::MaskCommand);

    // Post the key down event to HID event tap (system-level)
    CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&key_down_event));

    println!("[Auto-Paste] Posted Cmd+V key down event");

    // Small delay to ensure key down is processed before key up
    thread::sleep(Duration::from_millis(10));

    // Step 2: Create V key release event
    let key_up_event = CGEvent::new_keyboard_event(
        Some(&event_source),
        V_KEYCODE,
        false  // key up
    ).ok_or("Failed to create key up event")?;

    // Keep Command modifier during key up (some apps need this consistency)
    CGEvent::set_flags(Some(&key_up_event), CGEventFlags::MaskCommand);

    // Post the key up event
    CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&key_up_event));

    println!("[Auto-Paste] Posted Cmd+V key up event");

    Ok(())
}

/// Auto-paste text using Core Graphics events (Option 3)
///
/// This is an alternative to AppleScript that:
/// - ✅ Doesn't require System Events automation permissions
/// - ✅ Is faster (no process spawn overhead)
/// - ❓ Might not corrupt rdev's global state (needs testing)
///
/// This function:
/// 1. Saves the current clipboard content
/// 2. Sets the transcribed text to clipboard
/// 3. Simulates Cmd+V using Core Graphics events directly
/// 4. Restores the original clipboard after a delay
///
/// Returns Ok(()) on success, Err on clipboard or keyboard simulation failure
#[cfg(target_os = "macos")]
pub fn auto_paste_text_cgevent(text: &str) -> Result<(), String> {
    // Guard: Don't paste empty text
    if text.is_empty() {
        return Err("Cannot paste empty text".to_string());
    }

    println!("[Auto-Paste] Starting CGEvent-based auto-paste");

    // Step 1: Get clipboard instance
    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("Failed to access clipboard: {}", e))?;

    // Step 2: Save current clipboard content (if any)
    let previous_clipboard = clipboard.get_text().ok();
    if previous_clipboard.is_some() {
        println!("[Auto-Paste] Saved previous clipboard content");
    }

    // Step 3: Set transcribed text to clipboard
    clipboard
        .set_text(text.to_string())
        .map_err(|e| format!("Failed to set clipboard text: {}", e))?;

    println!("[Auto-Paste] Set clipboard to transcribed text ({} chars)", text.len());

    // Step 4: Simulate paste using Core Graphics
    paste_with_cgevent()?;

    println!("[Auto-Paste] ✅ CGEvent paste completed successfully");

    // Step 5: Restore previous clipboard content after a delay
    if let Some(previous_text) = previous_clipboard {
        let text_for_check = text.to_string();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(150)); // Wait for paste to complete

            // Best-effort restore with race condition protection
            if let Ok(mut clipboard) = Clipboard::new() {
                // Only restore if clipboard still contains our text
                // (avoid overwriting if user copied something else)
                if let Ok(current_text) = clipboard.get_text() {
                    if current_text == text_for_check {
                        if clipboard.set_text(previous_text).is_ok() {
                            println!("[Auto-Paste] Restored previous clipboard content");
                        }
                    } else {
                        println!("[Auto-Paste] Skipped clipboard restore (user copied new content)");
                    }
                }
            }
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_text_guard() {
        let result = auto_paste_text("");
        assert!(result.is_err());
    }
}
