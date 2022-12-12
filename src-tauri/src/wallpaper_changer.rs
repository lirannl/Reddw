use anyhow::{anyhow, Result};
use mime_guess::Mime;
use rand::seq::SliceRandom;
use reqwest::header::HeaderValue;
use serde::{Deserialize, Serialize};
use std::env::temp_dir;
use std::fs::DirEntry;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;
use std::{convert::Into, fs};
use tauri::async_runtime::{self, JoinHandle, Sender};
use tauri::{AppHandle, Manager};
use tokio::time::interval;
use ts_rs::TS;

use crate::app_config::{AppConfig, Source, CONFIG_PATH};
use crate::sources::reddit::get_from_subreddit;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub struct Wallpaper {
    pub id: String,
    pub info_url: String,
    pub source: Source,
    #[serde(skip)]
    pub data_url: String,
}

#[tauri::command]
pub async fn get_history() -> Result<Vec<Wallpaper>, String> {
    let config = crate::app_config::CONFIG.lock().await.clone();
    Ok(config.history)
}

async fn trim(config: &AppConfig) -> Result<()> {
    let mut files = fs::read_dir(&config.cache_dir)?
        .filter_map(|f| Some(f.ok()?))
        .collect::<Vec<DirEntry>>();
    let mut size: u64 = files
        .iter()
        .filter_map(|f| {
            if f.path().is_file() {
                Some(f.metadata().ok()?.len())
            } else {
                None
            }
        })
        .sum();
    if size > (config.cache_size * 1000000.0) as u64 {
        files.sort_by_key(|f| f.metadata().ok()?.created().ok());
        for file in files {
            if size < (config.cache_size * 1000000.0) as u64 {
                break;
            }
            size -= file.metadata()?.len();
            fs::remove_file(file.path())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn update_wallpaper(app_handle: AppHandle) -> Result<(), String> {
    let mut config = crate::app_config::CONFIG.lock().await.clone();
    if let Some(main_window) = app_handle.get_window("main") {
        main_window
            .emit("update_wallpaper_start", None::<()>)
            .map_err(|e| e.to_string())?;
    }
    let source = config
        .sources
        .choose(&mut rand::thread_rng())
        .ok_or("No sources")?;
    let wallpaper = match source {
        Source::Subreddit(subreddit) => get_from_subreddit(subreddit, &config).await,
    }
    .map_err(|e| format!("{:?}", e))?;
    // Add wallpaper to history
    config.history.push(wallpaper.clone());
    // Only save up to 50 items in history
    config.history = config.history.into_iter().rev().take(50).rev().collect();
    fs::write(
        CONFIG_PATH.lock().await.clone(),
        serde_json::to_string_pretty(&config).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    trim(&config).await.map_err(|err| err.to_string())?;
    {
        for wallpaper_file in fs::read_dir(&config.cache_dir)
            .map_err(|err| err.to_string())?
            .into_iter()
            .filter_map(|f| f.ok())
        {
            if (&config)
                .history
                .iter()
                .find(|h| wallpaper_file.file_name().to_str().unwrap() == &h.id)
                .is_some()
                .to_owned()
            {
                fs::remove_file(wallpaper_file.path()).unwrap();
            }
        }
    }
    let wp_req = reqwest::get(&wallpaper.data_url)
        .await
        .map_err(|err| err.to_string())?;
    let wallpaper_path = &config.cache_dir.join(format!(
        "{}.{}",
        &wallpaper.id,
        Mime::from_str(
            wp_req
                .headers()
                .get("Content-Type")
                .unwrap_or(&HeaderValue::from_static(""))
                .to_str()
                .map_err(|err| err.to_string())?
        )
        .map_err(|err| err.to_string())?
        .subtype()
        .as_str()
    ));
    fs::write(
        &wallpaper_path,
        wp_req.bytes().await.map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    wallpaper::set_from_path(wallpaper_path.to_str().ok_or("Wallpaper not downloaded")?)
        .map_err(|err| err.to_string())?;
    if let Some(main_window) = app_handle.get_window("main") {
        main_window
            .emit("update_wallpaper_stop", None::<()>)
            .map_err(|e| e.to_string())?;
    }
    println!("New wallpaper: {:#?}", wallpaper);
    Ok(())
}

pub fn setup_changer(app_handle: AppHandle) -> Sender<Duration> {
    let (tx_dur, mut rx_dur) = async_runtime::channel::<Duration>(100);
    let mut handle: Option<JoinHandle<_>> = None;
    async_runtime::spawn((async move || loop {
        let dur = rx_dur.recv().await.unwrap();
        if let Some(handle) = &handle {
            handle.abort();
        }
        if dur.is_zero() {
            println!("Updates disabled");
            continue;
        }
        let other_app_handle = app_handle.clone();
        handle = Some(async_runtime::spawn(async move {
            let mut interval = interval(dur);
            interval.tick().await;
            loop {
                interval.tick().await;
                update_wallpaper(other_app_handle.app_handle())
                    .await
                    .unwrap_or_else(|err| eprintln!("{:#?}", err));
            }
        }));
    })());
    tx_dur
}
