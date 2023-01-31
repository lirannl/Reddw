#![windows_subsystem = "windows"]
#![allow(incomplete_features)]
#![feature(async_closure, async_fn_in_trait, absolute_path, let_chains)]

mod app_config;
mod queue;
mod sources;
mod tray;
mod wallpaper_changer;

pub use anyhow::{anyhow, Result};
use queue::manage_queue;
use sysinfo::{ProcessExt, SystemExt};
use tauri::{async_runtime::block_on, generate_handler, AppHandle, Manager};
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

fn prevent_duplicate_proc() -> Result<(), String> {
    let sys = sysinfo::System::new_all();
    let this_pid = sysinfo::get_current_pid()?;
    let proc_name = sys
        .process(this_pid)
        .ok_or("Can't inspect this process")?
        .name();
    // Kill any other reddw instances (so that this one's in focus)
    for other_proc in sys
        .processes_by_name(proc_name)
        .filter(|&p| p.pid() != sysinfo::get_current_pid().unwrap())
    {
        other_proc.kill();
    }
    Ok(())
}

fn main() {
    prevent_duplicate_proc()
        .unwrap_or_else(|e| eprintln!("Error while preventing duplicate process: {e}"));
    tauri::Builder::default()
        .setup(|app| {
            main_window_setup(app.app_handle())?;
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
