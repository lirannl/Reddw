#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![feature(fs_try_exists)]
#![feature(io_error_more)]

mod db;
mod setup;

use setup::reddw_setup;
use std::error::Error;
use tauri::{async_runtime::block_on, AppHandle};

#[tauri::command]
fn get_path(handle: AppHandle) -> Result<String, String> {
    let appdir = handle
        .path_resolver()
        .app_dir()
        .ok_or("Couldn't get the app dir".to_string())?;
    let url = format!(
        "sqlite://{}?mode=rwc",
        appdir
            .join("reddw.db")
            .to_str()
            .ok_or("Coudln't resolve path".to_string())?
    );
    return Ok(url);
}

fn main() {
    tauri::Builder::default()
        .setup(|app| block_on(reddw_setup(app)).or_else(|e| Err(e as Box<dyn Error>)))
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
