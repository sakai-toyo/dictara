use crate::{
    audio_recorder::AudioRecorder,
    keyboard_listener::{start_fn_key_listener, FnKeyEvent},
    menu::build_menu,
    openai_client::OpenAIClient,
};
use cpal::traits::StreamTrait;
use std::sync::Mutex;
use tauri::{Emitter, Listener, Manager};

// Transcription event types
use serde::Serialize;

// Tray icon state for managing icon changes
pub struct TrayIconState {
    tray: Mutex<Option<tauri::tray::TrayIcon>>,
}

#[derive(Clone, Serialize)]
pub struct TranscriptionStartedEvent {
    pub timestamp: u128,
    pub filename: String,
}

#[derive(Clone, Serialize)]
pub struct TranscriptionCompletedEvent {
    pub timestamp: u128,
    pub filename: String,
    pub text: String,
    pub duration_ms: u64,
    pub char_count: usize,
}

#[derive(Clone, Serialize)]
pub struct TranscriptionErrorEvent {
    pub error: String,
    pub error_type: String,
    pub filename: String,
    pub timestamp: u128,
}

pub fn setup_app(app: &mut tauri::App<tauri::Wry>) -> Result<(), Box<dyn std::error::Error>> {
    // Check accessibility permission on macOS
    #[cfg(target_os = "macos")]
    {
        let has_permission = macos_accessibility_client::accessibility::application_is_trusted();
        if !has_permission {
            println!("⚠️  Accessibility permission not granted. Listener will fail.");
            // Frontend will handle permission request flow
        } else {
            println!("Accessibility is granted!")
        }
    }

    start_fn_key_listener(app.app_handle().clone());

    // Initialize OpenAI client
    let openai_client = match OpenAIClient::new() {
        Ok(client) => {
            println!("✅ OpenAI client initialized successfully");
            Some(client)
        }
        Err(e) => {
            eprintln!("⚠️  Failed to initialize OpenAI client: {}", e);
            eprintln!("    Transcription will be disabled.");
            eprintln!("    Set OPENAI_API_KEY in .env file to enable transcription.");
            None
        }
    };

    // Initialize audio recorder
    let recorder = AudioRecorder::new(app.app_handle().clone());

    // Since CPAL Stream is not Send, we leak it and manage via raw pointer
    // This is safe because we control when it's created/destroyed
    use std::ptr;
    use std::sync::atomic::{AtomicPtr, Ordering};

    static STREAM_PTR: AtomicPtr<cpal::Stream> = AtomicPtr::new(ptr::null_mut());

    let app_handle_for_transcription = app.app_handle().clone();

    // Subscribe to FN key events to control recording
    app.listen("fn-key-event", move |event| {
        let payload = event.payload();
        match serde_json::from_str::<FnKeyEvent>(payload) {
            Ok(fn_event) => {
                if fn_event.pressed {
                    println!("[Audio] FN key pressed - starting recording");
                    match recorder.start_recording() {
                        Ok(stream) => {
                            // Store stream via raw pointer (unsafe but controlled)
                            let stream_box = Box::new(stream);
                            let stream_ptr = Box::into_raw(stream_box);
                            STREAM_PTR.store(stream_ptr, Ordering::SeqCst);
                        }
                        Err(e) => {
                            eprintln!("[Audio] Start error: {:?}", e);
                            recorder.emit_error(&format!("{:?}", e), "start_error");
                        }
                    }
                } else {
                    println!("[Audio] FN key released - stopping recording");

                    // Retrieve and drop the stream
                    let stream_ptr = STREAM_PTR.swap(ptr::null_mut(), Ordering::SeqCst);
                    if !stream_ptr.is_null() {
                        unsafe {
                            let stream = Box::from_raw(stream_ptr);
                            stream.pause().ok();
                            drop(stream);
                        }
                    }

                    match recorder.stop_recording() {
                        Ok(()) => {
                            // Recording stopped successfully
                            // Now listen for the recording-stopped event to trigger transcription
                        }
                        Err(e) => {
                            eprintln!("[Audio] Stop error: {:?}", e);
                            recorder.emit_error(&format!("{:?}", e), "stop_error");
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[Audio] Failed to parse FN key event: {}", e);
            }
        }
    });

    // Subscribe to recording-stopped event to trigger transcription
    if let Some(client) = openai_client {
        use crate::audio_recorder::RecordingStoppedEvent;
        use std::path::PathBuf;
        use std::time::SystemTime;

        app.listen("recording-stopped", move |event| {
            let app_clone = app_handle_for_transcription.clone();

            // Parse the recording-stopped event
            let payload = event.payload();
            match serde_json::from_str::<RecordingStoppedEvent>(payload) {
                Ok(recording_event) => {
                    let filename = recording_event.filename.clone();
                    let duration_ms = recording_event.duration_ms;

                    println!(
                        "[Transcription] Recording stopped: {} ({}ms)",
                        filename, duration_ms
                    );

                    // Emit transcription started event
                    let timestamp = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_millis();

                    app_clone
                        .emit(
                            "transcription-started",
                            TranscriptionStartedEvent {
                                timestamp,
                                filename: filename.clone(),
                            },
                        )
                        .ok();

                    // Spawn async task to transcribe
                    let client_clone = client.clone();
                    let app_clone2 = app_clone.clone();

                    tauri::async_runtime::spawn(async move {
                        // Build file path
                        let file_path =
                            PathBuf::from("/Users/vitaliizinchenko/Projects/typefree/audio")
                                .join(&filename);

                        println!(
                            "[Transcription] Starting transcription for: {:?}",
                            file_path
                        );

                        // Transcribe the audio
                        match client_clone.transcribe_audio(file_path, duration_ms).await {
                            Ok(text) => {
                                let timestamp = SystemTime::now()
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis();

                                println!("[Transcription] ✅ Success: {}", text);

                                // AUTO-PASTE: Immediately paste the transcribed text
                                // Using CGEvent-based paste (Option 3) to avoid rdev state corruption
                                #[cfg(target_os = "macos")]
                                match crate::clipboard_paste::auto_paste_text_cgevent(&text) {
                                    Ok(()) => {
                                        println!("[Auto-Paste] ✅ Successfully pasted text: {}", text);
                                    }
                                    Err(e) => {
                                        eprintln!("[Auto-Paste] ⚠️  Failed to paste: {}", e);
                                    }
                                }

                                #[cfg(not(target_os = "macos"))]
                                {
                                    eprintln!("[Auto-Paste] ⚠️  Auto-paste not yet implemented for this platform");
                                }

                                app_clone2
                                    .emit(
                                        "transcription-completed",
                                        TranscriptionCompletedEvent {
                                            timestamp,
                                            filename: filename.clone(),
                                            text: text.clone(),
                                            duration_ms,
                                            char_count: text.len(),
                                        },
                                    )
                                    .ok();
                            }
                            Err(e) => {
                                let timestamp = SystemTime::now()
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis();

                                eprintln!("[Transcription] ❌ Error: {}", e);

                                let error_type = match e {
                                    crate::openai_client::TranscriptionError::AudioTooShort {
                                        ..
                                    } => "audio_too_short",
                                    crate::openai_client::TranscriptionError::FileTooLarge {
                                        ..
                                    } => "file_too_large",
                                    crate::openai_client::TranscriptionError::FileNotFound(_) => {
                                        "file_not_found"
                                    }
                                    crate::openai_client::TranscriptionError::ApiError(_) => {
                                        "api_error"
                                    }
                                    crate::openai_client::TranscriptionError::IoError(_) => {
                                        "io_error"
                                    }
                                    crate::openai_client::TranscriptionError::ApiKeyMissing => {
                                        "api_key_missing"
                                    }
                                };

                                app_clone2
                                    .emit(
                                        "transcription-error",
                                        TranscriptionErrorEvent {
                                            error: format!("{}", e),
                                            error_type: error_type.to_string(),
                                            filename,
                                            timestamp,
                                        },
                                    )
                                    .ok();
                            }
                        }
                    });
                }
                Err(e) => {
                    eprintln!(
                        "[Transcription] Failed to parse recording-stopped event: {}",
                        e
                    );
                }
            }
        });
    }

    let menu = build_menu(app)?;

    // Build tray icon
    let tray = tauri::tray::TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            match event.id().as_ref() {
                "about" => {
                    println!("About clicked - placeholder");
                    // TODO: Implement About dialog
                }
                "preferences" => {
                    println!("Preferences clicked - placeholder");
                    // TODO: Implement Preferences window
                }
                "quit" => {
                    println!("Quit clicked");
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    // Store tray icon in app state for dynamic icon updates
    let tray_state = TrayIconState {
        tray: Mutex::new(Some(tray)),
    };
    app.manage(tray_state);

    // Listen to recording events to change tray icon
    let app_handle_for_tray = app.app_handle().clone();
    app.listen("recording-started", move |_event| {
        println!("[Tray] Recording started - changing icon to red circle");
        if let Some(state) = app_handle_for_tray.try_state::<TrayIconState>() {
            if let Ok(tray_lock) = state.tray.lock() {
                if let Some(tray) = tray_lock.as_ref() {
                    // Load recording icon from embedded bytes
                    const RECORDING_ICON_BYTES: &[u8] = include_bytes!("../icons/recording.png");

                    // Decode PNG to RGBA
                    if let Ok(img) = image::load_from_memory(RECORDING_ICON_BYTES) {
                        let rgba = img.to_rgba8();
                        let (width, height) = rgba.dimensions();
                        let icon = tauri::image::Image::new_owned(rgba.into_raw(), width, height);

                        if let Err(e) = tray.set_icon(Some(icon)) {
                            eprintln!("[Tray] Failed to set recording icon: {}", e);
                        } else {
                            println!("[Tray] ✅ Icon changed to recording state");
                        }
                    } else {
                        eprintln!("[Tray] Failed to decode recording icon");
                    }
                }
            }
        }
    });

    let app_handle_for_tray2 = app.app_handle().clone();
    app.listen("recording-stopped", move |_event| {
        println!("[Tray] Recording stopped - changing icon back to default");
        if let Some(state) = app_handle_for_tray2.try_state::<TrayIconState>() {
            if let Ok(tray_lock) = state.tray.lock() {
                if let Some(tray) = tray_lock.as_ref() {
                    // Restore default icon
                    if let Some(default_icon) = app_handle_for_tray2.default_window_icon() {
                        if let Err(e) = tray.set_icon(Some(default_icon.clone())) {
                            eprintln!("[Tray] Failed to restore default icon: {}", e);
                        } else {
                            println!("[Tray] ✅ Icon restored to default state");
                        }
                    }
                }
            }
        }
    });

    Ok(())
}
