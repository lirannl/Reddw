#![feature(async_closure)]
mod app_config;
use tauri::{Manager, generate_handler, async_runtime::{spawn_blocking, block_on}};
#[allow(unused_imports)]
use window_vibrancy::{apply_acrylic, apply_vibrancy, Color};

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
        .invoke_handler(generate_handler![app_config::get_config])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
