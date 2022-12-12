#![feature(async_closure)]
#![feature(absolute_path)]
mod app_config;
mod sources;
mod tray;
mod wallpaper_changer;

use tauri::{
    async_runtime::{block_on},
    generate_handler, AppHandle, Manager,
};
use wallpaper_changer::setup_changer;

#[allow(unused_imports)]
use window_vibrancy::{apply_acrylic, apply_vibrancy, Color};

use crate::{
    app_config::{get_config, set_config},
    wallpaper_changer::{get_history, update_wallpaper},
};

fn main_window_setup(app: AppHandle) {
    let window = app.get_window("main").unwrap();
    #[cfg(target_os = "windows")]
    {
        apply_acrylic(window, None).unwrap();
    }
    #[cfg(target_os = "macos")]
    {
        apply_vibrancy(window, NSVisualEffectMaterial::HudWindow, None, None).unwrap();
    }

    app.get_window("main")
        .unwrap()
        .on_window_event(move |e| match e {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                app.app_handle()
                    .get_window("main")
                    .unwrap()
                    .hide()
                    .unwrap_or_else(|e| eprintln!("{:#?}", e));
            }
            _ => {}
        })
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let tx_interval = setup_changer(app.handle());
            main_window_setup(app.handle());
            // Setup config watcher
            block_on(app_config::build(app.handle(), tx_interval)).unwrap();
            // Setup wallpaper switcher
            Ok(())
        })
        .invoke_handler(generate_handler![
            get_config,
            set_config,
            update_wallpaper,
            get_history
        ])
        .system_tray(tray::setup())
        .on_system_tray_event(tray::event_handler)
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}
