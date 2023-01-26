#![windows_subsystem = "windows"]
#![feature(async_closure, absolute_path, async_fn_in_trait, let_chains)]

mod app_config;
mod queue;
mod sources;
mod tray;
mod wallpaper_changer;

pub use anyhow::anyhow;
use queue::manage_history;
use tauri::{async_runtime::block_on, generate_handler, AppHandle, Manager};
use wallpaper_changer::setup_changer;

#[allow(unused_imports)]
use window_vibrancy::{apply_acrylic, apply_vibrancy, Color};

use crate::{
    app_config::{get_config, set_config},
    wallpaper_changer::update_wallpaper,
};

fn main_window_setup(app: AppHandle) {
    let window =
        tauri::WindowBuilder::new(&app, "main", tauri::WindowUrl::App("index.html".into()))
            .title(env!("CARGO_PKG_NAME"))
            .build()
            .or_else(|_e| app.get_window("main").ok_or(anyhow!("Couldn't get window")))
            .unwrap();
    #[cfg(target_os = "windows")]
    {
        apply_acrylic(window, None).unwrap();
    }
    #[cfg(target_os = "macos")]
    {
        apply_vibrancy(window, NSVisualEffectMaterial::HudWindow, None, None).unwrap();
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            main_window_setup(app.app_handle());
            match app.get_cli_matches() {
                Err(e) => {
                    eprintln!("{:#?}", e);
                    std::process::exit(1);
                }
                Ok(matches) => {
                    if matches.args["background"].occurrences > 0 {
                        app.get_window("main")
                            .ok_or(anyhow!("No main window"))
                            .map(|w| w.close())??;
                    }
                }
            }
            let tx_interval = setup_changer(app.handle());
            // main_window_setup(app.handle());
            // Setup history
            block_on(manage_history(app.handle())).unwrap();
            // Setup config watcher
            block_on(app_config::build(app.handle(), tx_interval)).unwrap();
            // Setup wallpaper switcher
            Ok(())
        })
        .invoke_handler(generate_handler![
            get_config,
            set_config,
            update_wallpaper,
            // get_history,
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
