#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![feature(fs_try_exists)]

use setup::reddw_setup;

mod db;
mod setup;

fn main() {
    tauri::Builder::default()
        .setup(reddw_setup)
        .invoke_handler(tauri::generate_handler![db::check_config])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
