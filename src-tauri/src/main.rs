#![windows_subsystem = "windows"]
#![allow(incomplete_features)]
#![feature(async_closure, absolute_path, let_chains, if_let_guard)]

mod app_config;
mod app_handle_ext;
mod automation_socket;
mod log;
mod queue;
mod source_host;
mod tray;
mod wallpaper_changer;
use crate::{
    app_config::{get_config, select_folder, set_config},
    queue::{cache_queue, get_queue},
    source_host::{load_plugin_ui, query_available_source_plugins},
    wallpaper_changer::{get_wallpaper, set_wallpaper, update_wallpaper},
};
use anyhow::{anyhow, Result};
use automation_socket::Args;
use clap::Parser;
use queue::manage_queue;
use tauri::{async_runtime::block_on, generate_handler, AppHandle, Manager, Window};
use wallpaper_changer::setup_changer;
#[cfg(target_os = "windows")]
use window_vibrancy::apply_acrylic;
#[cfg(target_os = "macos")]
use window_vibrancy::{apply_vibrancy, Color};

fn main_window_setup(app: AppHandle) -> Result<Window> {
    let window =
        tauri::WindowBuilder::new(&app, "main", tauri::WindowUrl::App("index.html".into()))
            .title(env!("CARGO_PKG_NAME"))
            .transparent(true)
            .visible(false)
            .build()
            .or_else(|_e| app.get_window("main").ok_or(anyhow!("Couldn't get window")))?;
    #[cfg(development)]
    window.open_devtools();
    #[cfg(target_os = "windows")]
    {
        apply_acrylic(&window, None).map_err(|e| anyhow!(e))?;
    }
    #[cfg(target_os = "macos")]
    {
        apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, None)
            .map_err(|e| anyhow!(e))?;
    }
    Ok(window)
}

#[tauri::command]
fn exit() {
    std::process::exit(0);
}

fn main() {
    let args = Args::parse();
    tauri::Builder::default()
        .setup(move |app| {
            main_window_setup(app.app_handle())?;
            block_on(automation_socket::initiate_ipc(&args, app.app_handle()))?;
            if args.background {
                app.get_window("main")
                    .ok_or(anyhow!("No main window"))
                    .map(|w| w.close())??;
            } else {
                app.get_window("main")
                    .ok_or(anyhow!("No main window"))
                    .map(|w| w.show())??;
            };
            let tx_interval = setup_changer(app.handle());
            match {
                // Setup config watcher
                app_config::build(app.handle(), tx_interval)?;
                // Setup history
                block_on(manage_queue(&app.handle()))?;
                Result::<(), anyhow::Error>::Ok(())
            } {
                Ok(()) => (),
                Err(e) => {
                    eprintln!("Error while setting up {e:#?}");
                    std::process::exit(1);
                }
            }
            block_on(source_host::host_plugins(app.handle()))?;
            Ok(())
        })
        .invoke_handler(generate_handler![
            get_config,
            set_config,
            update_wallpaper,
            query_available_source_plugins,
            load_plugin_ui,
            cache_queue,
            get_queue,
            select_folder,
            set_wallpaper,
            get_wallpaper,
            exit,
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
