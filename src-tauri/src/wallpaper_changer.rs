use crate::app_config::{AppConfig, Source};
use crate::queue::DB;
use crate::sources::reddit::get_from_subreddit;
use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use data_encoding::BASE32;
use mime_guess::{Mime, MimeGuess};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{query_as, FromRow};
use std::fs::{self, DirEntry};
use std::path::{Path, self};
use std::str::FromStr;
use std::time::Duration;
use tauri::utils::debug_eprintln;
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
    pub name: String,
    pub data_url: String,
    pub info_url: Option<String>,
    #[ts(type = "string")]
    pub date: NaiveDateTime,
    pub source: String,
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
    let config = app_handle.state::<AppConfig>();
    let source = config
        .sources
        .choose(&mut rand::thread_rng())
        .ok_or(anyhow!("No sources"))?;

    let source_str = serde_json::to_string(source)?;
    let app_handle_clone = app_handle.clone();
    let get_wp = async || {
        query_as!(
            Wallpaper,
            "---sql
            select * from queue 
            where source = $1 and was_set = 0",
            source_str
        )
        .fetch_optional(&mut app_handle_clone.state::<DB>().acquire().await?)
        .await?
        .ok_or(anyhow!("No wallpapers"))
    };

    let wp = get_wp().await;
    let app_handle_clone = app_handle.clone();

    let wallpaper = match wp {
        Ok(wallpaper) => Ok::<Wallpaper, anyhow::Error>(wallpaper),
        Err(e) => {
            let e = e as anyhow::Error;
            if format!("{e}").contains("No wallpapers") {
                match source {
                    Source::Subreddit(sub) => get_from_subreddit(sub, &app_handle_clone).await?,
                }
            };
            let wallpaper = get_wp().await?;
            Ok(wallpaper)
        }
    }?;

    let wallpaper_path = || {
        app_handle.state::<AppConfig>().cache_dir.join(&wallpaper.id)
    };
    
    wallpaper::set_from_path().map_err(|e| anyhow!(e.to_string()))?;
    if let Some(main_window) = app_handle.get_window("main") {
        main_window.emit("update_wallpaper_stop", None::<()>)?;
    }
    app_handle.tray_handle().get_item("update_wallpaper").set_title(wallpaper.name.as_str())?;
    eprintln!("New wallpaper: {:#?}", wallpaper);
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
                update_wallpaper_internal(other_app_handle.app_handle())
                    .await
                    .unwrap_or_else(|err| eprintln!("{:#?}", err));
            }
        }));
    })());
    tx_dur
}

pub async fn download_wallpaper(app_handle: &AppHandle, wp_url: String) -> Result<()> {
    let wp_res = reqwest::get(&wp_url).await?;
    let wallpaper_filename = format!(
        "{}.{}",
        &BASE32.encode(&Sha256::digest(&wp_url.as_bytes()))[..7],
        Mime::from_str(
            wp_res
                .headers()
                .get("Content-Type")
                .ok_or(anyhow!("Couldn't determine extension"))?
                .to_str()?
        )?
        .subtype()
        .as_str()
    );
    let cache_folder = app_handle
        .path_resolver()
        .app_cache_dir()
        .ok_or(anyhow!("No cache folder"))?;
    debug_eprintln!("Downloading {} to {}", wp_url, wallpaper_filename);
    fs::write(
        &cache_folder.join(wallpaper_filename),
        wp_res.bytes().await?,
    )?;
    Ok(())
}

#[tauri::command]
pub async fn update_wallpaper(app_handle: AppHandle) -> Result<(), String> {
    if let Some(main_window) = app_handle.get_window("main") {
        main_window
            .emit("update_wallpaper_start", None::<()>)
            .map_err(|e| e.to_string())?;
    }
    let res = update_wallpaper_internal(app_handle.app_handle()).await;
    if let Some(main_window) = app_handle.get_window("main") {
        main_window
            .emit("update_wallpaper_stop", None::<()>)
            .map_err(|e| e.to_string())?;
    }
    res.map_err(|e| {
        eprintln!("{e:#?}");
        e.to_string()
    })
}
