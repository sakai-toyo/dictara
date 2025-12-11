mod audio_recorder;
mod commands;
mod keyboard_listener;
mod menu;
mod openai_client;
mod setup;

pub fn run() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    tauri::Builder::default()
        .setup(|app| {
            return setup::setup_app(app);
        })
        .invoke_handler(tauri::generate_handler![
            commands::check_accessibility_permission,
            commands::request_accessibility_permission,
            commands::restart_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
