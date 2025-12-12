use tauri::{App, Wry};

pub fn build_menu(app: &App<Wry>) -> Result<tauri::menu::Menu<Wry>, Box<dyn std::error::Error>> {
    // Build menu items
    let preferences_item =
        tauri::menu::MenuItemBuilder::with_id("preferences", "Preferences").build(app)?;
    let quit_item = tauri::menu::MenuItemBuilder::with_id("quit", "Quit").build(app)?;

    // Build menu
    let menu = tauri::menu::MenuBuilder::new(app)
        .item(&preferences_item)
        .separator()
        .item(&quit_item)
        .build()?;

    Ok(menu)
}
