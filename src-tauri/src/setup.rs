use crate::{
    clients::openai::OpenAIClient,
    keyboard_listener::KeyListener,
    recording::{Controller, RecordingCommand},
    ui::{menu::build_menu, tray::TrayIconState, window},
};
use std::sync::{atomic::AtomicU8, Arc, Mutex};
use tauri::Manager;
use tokio::sync::mpsc;

pub struct RecordingCommandSender {
    pub sender: mpsc::Sender<RecordingCommand>,
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

    #[cfg(target_os = "macos")]
    {
        // Keep the app running in the background
        app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    }

    // Initialize OpenAI client (always succeeds, key checked at transcription time)
    let openai_client = OpenAIClient::new();

    // Check if API key is configured
    let needs_api_key = !OpenAIClient::has_api_key();
    if needs_api_key {
        println!("⚠️  No OpenAI API key configured. Opening Preferences...");
    } else {
        println!("✅ OpenAI client initialized successfully");
    }

    // ========================================
    // CHANNEL-BASED ARCHITECTURE WITH CONTROLLER
    // Setup creates the channel and wires components together
    // ========================================

    // Create channel for recording commands (KeyListener → Controller)
    let (command_tx, command_rx) = mpsc::channel::<RecordingCommand>(100);
    let recording_state = Arc::new(AtomicU8::new(0));

    // Clone sender for Tauri state (mpsc::Sender is Clone + Send + Sync)
    let command_sender_state = RecordingCommandSender {
        sender: command_tx.clone(),
    };

    // Initialize controller with OpenAI client
    let controller = Controller::new(
        command_rx,
        app.app_handle().clone(),
        openai_client,
        recording_state.clone(),
    );

    // Spawn controller in blocking thread (cpal::Stream is not Send)
    std::thread::spawn(move || {
        controller.run();
    });

    // Store sender in app state for Tauri commands
    app.manage(command_sender_state);

    // Start keyboard listener with command sender
    let _listener = KeyListener::start(command_tx, recording_state);

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
                    println!("Preferences clicked");
                    if let Err(e) = window::open_preferences_window(app) {
                        eprintln!("Failed to open preferences window: {}", e);
                    }
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

    // Open preferences window if no API key is configured
    if needs_api_key {
        if let Err(e) = window::open_preferences_window(&app.app_handle()) {
            eprintln!("Failed to open preferences window: {}", e);
        }
    }

    Ok(())
}
