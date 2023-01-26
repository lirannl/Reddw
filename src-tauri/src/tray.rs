use tauri::{
    async_runtime, AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
};

use crate::{
    app_config::CONFIG, queue::Queue, main_window_setup, wallpaper_changer::update_wallpaper,
};

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
                    let id = id.clone();
                    let handle = app.app_handle();
                    let item_handle = handle.tray_handle().get_item(&id);
                    async_runtime::spawn(async move {
                        item_handle
                            .set_enabled(false)
                            .unwrap_or_else(|e| eprintln!("{:#?}", e));
                        item_handle
                            .set_title("Updating...")
                            .unwrap_or_else(|e| eprintln!("{:#?}", e));
                        update_wallpaper(handle)
                            .await
                            .unwrap_or_else(|e| eprintln!("{:#?}", e));
                        item_handle
                            .set_title("Update Wallpaper")
                            .unwrap_or_else(|e| eprintln!("{:#?}", e));
                        item_handle
                            .set_enabled(true)
                            .unwrap_or_else(|e| eprintln!("{:#?}", e));
                    });
                }
                "open_info" => {
                    // async_runtime::spawn(async {
                    //     let config = &*CONFIG.lock().await;
                    //     if let Some((_, wp)) = app.state::<History>().lock().await.last() {
                    //         open::that(&wp.info_url).unwrap_or_else(|e| eprintln!("{:#?}", e));
                    //     }
                    // });
                }
                "show" => {
                    if let Some(w) = app.get_window("main") {
                        w.set_focus().unwrap_or(());
                    } else {
                        main_window_setup(app.app_handle());
                    }
                }
                "quit" => app.exit(0),
                _ => {}
            }
        }
        _ => {}
    }
}
