use tauri::{Manager, Listener, Emitter};
use cpal::traits::StreamTrait;

mod keyboard_listener;
mod audio_recorder;
mod openai_client;

use crate::keyboard_listener::{start_fn_key_listener, FnKeyEvent};
use crate::audio_recorder::AudioRecorder;
use crate::openai_client::OpenAIClient;

// Transcription event types
use serde::Serialize;

#[derive(Clone, Serialize)]
struct TranscriptionStartedEvent {
    timestamp: u128,
    filename: String,
}

#[derive(Clone, Serialize)]
struct TranscriptionCompletedEvent {
    timestamp: u128,
    filename: String,
    text: String,
    duration_ms: u64,
    char_count: usize,
}

#[derive(Clone, Serialize)]
struct TranscriptionErrorEvent {
    error: String,
    error_type: String,
    filename: String,
    timestamp: u128,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn check_accessibility_permission() -> bool {
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
fn request_accessibility_permission() {
    #[cfg(target_os = "macos")]
    {
        // This will show macOS system dialog and open System Settings
        macos_accessibility_client::accessibility::application_is_trusted_with_prompt();
    }
}

#[tauri::command]
fn restart_app(app: tauri::AppHandle) {
    app.restart();
}

pub fn run() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    tauri::Builder::default()
        .setup(|app| {
            // Check accessibility permission on macOS
            #[cfg(target_os = "macos")]
            {
                let has_permission =
                    macos_accessibility_client::accessibility::application_is_trusted();
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
            use std::sync::atomic::{AtomicPtr, Ordering};
            use std::ptr;

            static STREAM_PTR: AtomicPtr<cpal::Stream> = AtomicPtr::new(ptr::null_mut());

            let recorder_clone = recorder.clone();
            let app_handle_for_transcription = app.app_handle().clone();

            // Subscribe to FN key events to control recording
            app.listen("fn-key-event", move |event| {
                let payload = event.payload();
                match serde_json::from_str::<FnKeyEvent>(payload) {
                    Ok(fn_event) => {
                        if fn_event.pressed {
                            println!("[Audio] FN key pressed - starting recording");
                            match recorder_clone.start_recording() {
                                Ok(stream) => {
                                    // Store stream via raw pointer (unsafe but controlled)
                                    let stream_box = Box::new(stream);
                                    let stream_ptr = Box::into_raw(stream_box);
                                    STREAM_PTR.store(stream_ptr, Ordering::SeqCst);
                                }
                                Err(e) => {
                                    eprintln!("[Audio] Start error: {:?}", e);
                                    recorder_clone.emit_error(&format!("{:?}", e), "start_error");
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

                            match recorder_clone.stop_recording() {
                                Ok(()) => {
                                    // Recording stopped successfully
                                    // Now listen for the recording-stopped event to trigger transcription
                                }
                                Err(e) => {
                                    eprintln!("[Audio] Stop error: {:?}", e);
                                    recorder_clone.emit_error(&format!("{:?}", e), "stop_error");
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
                use std::time::SystemTime;
                use std::path::PathBuf;

                app.listen("recording-stopped", move |event| {
                    let app_clone = app_handle_for_transcription.clone();

                    // Parse the recording-stopped event
                    let payload = event.payload();
                    match serde_json::from_str::<RecordingStoppedEvent>(payload) {
                        Ok(recording_event) => {
                            let filename = recording_event.filename.clone();
                            let duration_ms = recording_event.duration_ms;

                            println!("[Transcription] Recording stopped: {} ({}ms)", filename, duration_ms);

                            // Emit transcription started event
                            let timestamp = SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_millis();

                            app_clone.emit(
                                "transcription-started",
                                TranscriptionStartedEvent {
                                    timestamp,
                                    filename: filename.clone(),
                                }
                            ).ok();

                            // Spawn async task to transcribe
                            let client_clone = client.clone();
                            let app_clone2 = app_clone.clone();

                            tauri::async_runtime::spawn(async move {
                                // Build file path
                                let file_path = PathBuf::from("/Users/vitaliizinchenko/Projects/typefree/audio")
                                    .join(&filename);

                                println!("[Transcription] Starting transcription for: {:?}", file_path);

                                // Transcribe the audio
                                match client_clone.transcribe_audio(file_path, duration_ms).await {
                                    Ok(text) => {
                                        let timestamp = SystemTime::now()
                                            .duration_since(SystemTime::UNIX_EPOCH)
                                            .unwrap()
                                            .as_millis();

                                        println!("[Transcription] ✅ Success: {}", text);

                                        app_clone2.emit(
                                            "transcription-completed",
                                            TranscriptionCompletedEvent {
                                                timestamp,
                                                filename: filename.clone(),
                                                text: text.clone(),
                                                duration_ms,
                                                char_count: text.len(),
                                            }
                                        ).ok();
                                    }
                                    Err(e) => {
                                        let timestamp = SystemTime::now()
                                            .duration_since(SystemTime::UNIX_EPOCH)
                                            .unwrap()
                                            .as_millis();

                                        eprintln!("[Transcription] ❌ Error: {}", e);

                                        let error_type = match e {
                                            crate::openai_client::TranscriptionError::AudioTooShort { .. } => "audio_too_short",
                                            crate::openai_client::TranscriptionError::FileTooLarge { .. } => "file_too_large",
                                            crate::openai_client::TranscriptionError::FileNotFound(_) => "file_not_found",
                                            crate::openai_client::TranscriptionError::ApiError(_) => "api_error",
                                            crate::openai_client::TranscriptionError::IoError(_) => "io_error",
                                            crate::openai_client::TranscriptionError::ApiKeyMissing => "api_key_missing",
                                        };

                                        app_clone2.emit(
                                            "transcription-error",
                                            TranscriptionErrorEvent {
                                                error: format!("{}", e),
                                                error_type: error_type.to_string(),
                                                filename,
                                                timestamp,
                                            }
                                        ).ok();
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            eprintln!("[Transcription] Failed to parse recording-stopped event: {}", e);
                        }
                    }
                });
            }

            // Store recorder in app state for potential future commands
            app.manage(recorder);

            // Build menu items
            let about_item = tauri::menu::MenuItemBuilder::with_id("about", "About").build(app)?;
            let preferences_item =
                tauri::menu::MenuItemBuilder::with_id("preferences", "Preferences").build(app)?;
            let quit_item = tauri::menu::MenuItemBuilder::with_id("quit", "Quit").build(app)?;

            // Build menu
            let menu = tauri::menu::MenuBuilder::new(app)
                .item(&about_item)
                .item(&preferences_item)
                .separator()
                .item(&quit_item)
                .build()?;

            // Build tray icon
            let _tray = tauri::tray::TrayIconBuilder::new()
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

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            check_accessibility_permission,
            request_accessibility_permission,
            // start_fn_listener,
            restart_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
