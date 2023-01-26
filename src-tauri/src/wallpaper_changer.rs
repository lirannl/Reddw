use crate::app_config::{AppConfig, Source};
use crate::queue::{Queue, DB};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use sqlx::{encode, query, query_as, Executor, FromRow};
use std::fs::{self, DirEntry};
use std::path::Path;
use std::time::Duration;
use tauri::api::path::cache_dir;
use tauri::{
    async_runtime::{self, JoinHandle, Sender},
    AppHandle, Manager,
};
use tokio::time::interval;
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, TS, FromRow)]
#[ts(export)]
pub struct Wallpaper {
    pub id: String,
    pub file_name: String,
    #[ts(type = "string")]
    pub date: DateTime<Utc>,
    pub info_url: String,
    pub source: Source,
    pub was_set: bool,
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

async fn update_wallpaper_internal(app_handle: AppHandle) -> Result<()> {
    let config = crate::app_config::CONFIG.lock().await.clone();
    if let Some(main_window) = app_handle.get_window("main") {
        main_window.emit("update_wallpaper_start", None::<()>)?;
    }
    let source = config
        .sources
        .choose(&mut rand::thread_rng())
        .ok_or(anyhow!("No sources"))?;

    let source_str = serde_json::to_string(source)?;
    let wallpaper: Wallpaper =
        query_as("SELECT * FROM queue WHERE source = ? AND was_set = 0 ORDER BY date DESC LIMIT 1")
            .bind(source_str)
            .fetch_one(&mut app_handle.state::<DB>().acquire().await?).await?
            .await?;

    wallpaper::set_from_path(
        Path::join(
            &cache_dir().ok_or(anyhow!("No cache dir"))?,
            wallpaper.file_name,
        )
        .to_str()
        .ok_or(anyhow!("Wallpaper not downloaded"))?,
    ).map_err(|e| anyhow!(e.to_string()))?;
    if let Some(main_window) = app_handle.get_window("main") {
        main_window.emit("update_wallpaper_stop", None::<()>)?;
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

#[tauri::command]
pub async fn update_wallpaper(app_handle: AppHandle) -> Result<(), String> {
    update_wallpaper_internal(app_handle)
        .await
        .map_err(|e| e.to_string())
}
