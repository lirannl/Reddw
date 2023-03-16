#![windows_subsystem = "windows"]
#![allow(incomplete_features)]
#![feature(async_closure, async_fn_in_trait, absolute_path, let_chains, if_let_guard)]

mod app_config;
mod automation_socket;
mod queue;
mod sources;
mod tray;
mod wallpaper_changer;

use anyhow::{Result, anyhow};
use queue::manage_queue;
use tauri::{api::cli, async_runtime::block_on, generate_handler, AppHandle, Manager};
use wallpaper_changer::setup_changer;

#[allow(unused_imports)]
use window_vibrancy::{apply_acrylic, apply_vibrancy, Color};

use crate::{
    app_config::{get_config, set_config},
    queue::{cache_queue, get_queue},
    wallpaper_changer::update_wallpaper,
};

fn main_window_setup(app: AppHandle) -> Result<()> {
    let window =
        tauri::WindowBuilder::new(&app, "main", tauri::WindowUrl::App("index.html".into()))
            .title(env!("CARGO_PKG_NAME"))
            .build()
            .or_else(|_e| app.get_window("main").ok_or(anyhow!("Couldn't get window")))?;
    #[cfg(target_os = "windows")]
    {
        apply_acrylic(window, None).map_err(|e| anyhow!(e))?;
    }
    #[cfg(target_os = "macos")]
    {
        apply_vibrancy(window, NSVisualEffectMaterial::HudWindow, None, None)
            .map_err(|e| anyhow!(e))?;
    }
    Ok(())
}



#[tauri::command]
fn exit()
{
    std::process::exit(0);
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(app.get_cli_matches().unwrap());
            main_window_setup(app.app_handle())?;
            block_on(automation_socket::participate(app.app_handle()))?;
            if app.state::<cli::Matches>().args["background"].occurrences > 0 {
                app.get_window("main")
                    .ok_or(anyhow!("No main window"))
                    .map(|w| w.close())??;
            };
            let tx_interval = setup_changer(app.handle());
            match {
                // Setup config watcher
                block_on(app_config::build(app.handle(), tx_interval))?;
                // Setup history
                block_on(manage_queue(app.handle()))?;
                Result::<(), anyhow::Error>::Ok(())
            } {
                Ok(()) => (),
                Err(e) => {
                    eprintln!("Error while setting up {e:#?}");
                    std::process::exit(1);
                }
            }
            Ok(())
        })
        .invoke_handler(generate_handler![
            get_config,
            set_config,
            update_wallpaper,
            cache_queue,
            get_queue,
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
