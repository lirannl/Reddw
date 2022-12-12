use anyhow::Result;
use tauri::{AppHandle, CustomMenuItem, SystemTray, SystemTrayMenu, WindowEvent, SystemTrayEvent, async_runtime, Manager};

use crate::{wallpaper_changer::update_wallpaper, app_config::CONFIG};

pub fn setup() -> SystemTray {
    let tray = SystemTray::new().with_menu(
        SystemTrayMenu::new()
            .add_item(CustomMenuItem::new("update_wallpaper", "Update Wallpaper"))
            .add_item(CustomMenuItem::new("open_info", "Show information"))
            .add_native_item(tauri::SystemTrayMenuItem::Separator)
            .add_item(CustomMenuItem::new("show", "Show"))
            .add_item(CustomMenuItem::new("quit", "Quit")),
    );
    tray
}

pub fn event_handler(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::MenuItemClick { id, .. } => {
            // let item_handle = app.tray_handle().get_item(&id);
            match id.as_str() {
                "update_wallpaper" => {
                    async_runtime::spawn(update_wallpaper(app.app_handle()));
                }
                "open_info" => {
                    async_runtime::spawn(async {
                        let config = &*CONFIG.lock().await;
                        if let Some(current) = config.history.last()
                        {
                            open::that(&current.info_url).unwrap_or_else(|e| eprintln!("{:#?}", e));
                        }
                    });
                }
                "show" => {
                    app.get_window("main")
                        .unwrap()
                        .show()
                        .unwrap_or_else(|e| eprintln!("{:#?}", e));
                }
                "quit" => app.exit(0),
                _ => {}
            }
        }
        _ => {}
    }
}
