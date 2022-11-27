#![feature(async_closure)]
mod app_config;
use tauri::{
    async_runtime::{block_on, spawn_blocking},
    generate_handler, Manager,
};
#[allow(unused_imports)]
use window_vibrancy::{apply_acrylic, apply_vibrancy, Color};

use crate::app_config::{get_config, set_config};

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(target_os = "windows")]
            {
                let window = app.handle().get_window("main").unwrap();
                apply_acrylic(window, None).unwrap();
            }
            #[cfg(target_os = "macos")]
            {
                let window = app.handle().get_window("main").unwrap();
                apply_vibrancy(window, NSVisualEffectMaterial::HudWindow, None, None).unwrap();
            }
            block_on(app_config::build(app.handle())).unwrap();
            Ok(())
        })
        .invoke_handler(generate_handler![get_config, set_config])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
