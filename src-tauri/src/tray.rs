use crate::{
    app_handle_ext::AppHandleExt,
    main_window_setup,
    wallpaper_changer::update_wallpaper,
};
use reddw_source_plugin::Wallpaper;
use sqlx::query_as;
use tauri::{
    async_runtime, AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
};

pub fn setup() -> SystemTray {
    let tray = SystemTray::new().with_menu(
        SystemTrayMenu::new()
            .add_item(CustomMenuItem::new("open_info", "Show information"))
            .add_item(CustomMenuItem::new("update_wallpaper", "Update Wallpaper"))
            .add_native_item(tauri::SystemTrayMenuItem::Separator)
            .add_item(CustomMenuItem::new("show", "Show"))
            .add_item(CustomMenuItem::new("quit", "Quit")),
    );
    tray
}

pub fn event_handler(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick { .. } => {
            if app.get_window("main").is_none() {
                main_window_setup(app.app_handle())
                    .map(|_w| {})
                    .unwrap_or_else(|e| eprintln!("{e:#?}"));
            };
        }
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
                    let app_clone = app.app_handle();
                    async_runtime::spawn(async move {
                        let dbconn = app_clone.db().await;
                        let info_url = query_as!(
                            Wallpaper,
                            "---sql
                            select * from queue 
                            where was_set = 1
                            order by date desc",
                        )
                        .fetch_optional(&dbconn)
                        .await?
                        .and_then(|a| a.info_url);
                        if let Some(info_url) = info_url {
                            open::that(&info_url).unwrap_or_else(|e| eprintln!("{:#?}", e));
                        }
                        Ok::<_, anyhow::Error>(())
                    });
                }
                "show" => {
                    if let Some(w) = app.get_window("main") {
                        w.set_focus().unwrap_or(());
                    } else {
                        main_window_setup(app.app_handle())
                            .and_then(|w| w.show().map_err(|e| anyhow::anyhow!(e)))
                            .unwrap_or_else(|e| eprintln!("{e:#?}"));
                    }
                }
                "quit" => app.exit(0),
                _ => {}
            }
        }
        _ => {}
    }
}
