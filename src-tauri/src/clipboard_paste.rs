use arboard::Clipboard;
use std::{thread, time::Duration};

#[cfg(target_os = "macos")]
use objc2_core_graphics::{
    CGEvent, CGEventFlags, CGEventSource, CGEventSourceStateID, CGEventTapLocation, CGKeyCode,
};

#[derive(Debug)]
pub enum ClipboardPasteError {
    EventSourceCreationFailed,
    KeyEventCreationFailed,
    EmptyText,
    ClipboardAccessFailed(String),
    ClipboardSetFailed(String),
    #[cfg(not(target_os = "macos"))]
    UnsupportedPlatform,
}

impl std::fmt::Display for ClipboardPasteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipboardPasteError::EventSourceCreationFailed => {
                write!(f, "Failed to create Core Graphics event source")
            }
            ClipboardPasteError::KeyEventCreationFailed => {
                write!(f, "Failed to create keyboard event")
            }
            ClipboardPasteError::EmptyText => {
                write!(f, "Cannot paste empty text")
            }
            ClipboardPasteError::ClipboardAccessFailed(msg) => {
                write!(f, "Failed to access clipboard: {}", msg)
            }
            ClipboardPasteError::ClipboardSetFailed(msg) => {
                write!(f, "Failed to set clipboard text: {}", msg)
            }
            #[cfg(not(target_os = "macos"))]
            ClipboardPasteError::UnsupportedPlatform => {
                write!(f, "Auto-paste not yet implemented for this platform")
            }
        }
    }
}

/// Returns Ok(()) on success, Err on event creation/posting failure
#[cfg(target_os = "macos")]
pub fn paste_with_cgevent() -> Result<(), ClipboardPasteError> {
    // Key code for 'V' key on macOS keyboard
    const V_KEYCODE: CGKeyCode = 9;

    println!("[Auto-Paste] Using Core Graphics to simulate Cmd+V");

    // Create event source for HID system state
    let event_source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .ok_or(ClipboardPasteError::EventSourceCreationFailed)?;

    // Step 1: Create V key press event with Command modifier
    let key_down_event = CGEvent::new_keyboard_event(
        Some(&event_source),
        V_KEYCODE,
        true, // key down
    )
    .ok_or(ClipboardPasteError::KeyEventCreationFailed)?;

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
        false, // key up
    )
    .ok_or(ClipboardPasteError::KeyEventCreationFailed)?;

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
pub fn auto_paste_text_cgevent(text: &str) -> Result<(), ClipboardPasteError> {
    // Guard: Don't paste empty text
    if text.is_empty() {
        return Err(ClipboardPasteError::EmptyText);
    }

    println!("[Auto-Paste] Starting CGEvent-based auto-paste");

    // Step 1: Get clipboard instance
    let mut clipboard =
        Clipboard::new().map_err(|e| ClipboardPasteError::ClipboardAccessFailed(e.to_string()))?;

    // Step 2: Save current clipboard content (if any)
    let previous_clipboard = clipboard.get_text().ok();
    if previous_clipboard.is_some() {
        println!("[Auto-Paste] Saved previous clipboard content");
    }

    // Step 3: Set transcribed text to clipboard
    clipboard
        .set_text(text.to_string())
        .map_err(|e| ClipboardPasteError::ClipboardSetFailed(e.to_string()))?;

    println!(
        "[Auto-Paste] Set clipboard to transcribed text ({} chars)",
        text.len()
    );

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
                        println!(
                            "[Auto-Paste] Skipped clipboard restore (user copied new content)"
                        );
                    }
                }
            }
        });
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn auto_paste_text_cgevent(text: &str) -> Result<(), ClipboardPasteError> {
    eprintln!("[Auto-Paste] Auto-paste not yet implemented for this platform");
    Err(ClipboardPasteError::UnsupportedPlatform)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_text_guard() {
        let result = auto_paste_text_cgevent("");
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, ClipboardPasteError::EmptyText));
        }
    }
}
