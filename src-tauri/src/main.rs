use tauri::Manager;
use window_vibrancy::{apply_acrylic, Color, apply_vibrancy};

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(target_os = "windows")]
            {
                let window = app.handle().get_window("main").unwrap();
                apply_acrylic(window, None);
            }
            #[cfg(target_os = "macos")]
            {
                let window = app.handle().get_window("main").unwrap();
                apply_vibrancy(window, NSVisualEffectMaterial::HudWindow, None, None)
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
