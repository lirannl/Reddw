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
use tauri::async_runtime::block_on;

use crate::db::{add_source, get_sources, remove_source};

fn main() {
    tauri::Builder::default()
        .setup(|app| block_on(reddw_setup(app)).or_else(|e| Err(e as Box<dyn Error>)))
        .invoke_handler(tauri::generate_handler![get_sources, add_source, remove_source])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
