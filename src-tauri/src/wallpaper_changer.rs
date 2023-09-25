use crate::app_config::Source;
use crate::app_handle_ext::AppHandleExt;
use crate::queue::trim_queue;
use crate::sources::reddit::get_from_subreddit;
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine};
use chrono::NaiveDateTime;
use data_encoding::BASE32;
use mime_guess::mime::IMAGE;
use mime_guess::Mime;
use rand::seq::SliceRandom;
use reddw_source_plugin::Wallpaper;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{query, query_as, FromRow};
use std::fmt::Display;
use std::fs::{self, read_dir as read_dir_sync};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use tauri::{
    async_runtime::{self, JoinHandle, Sender},
    AppHandle, Manager,
};
use tokio::{fs::read, time::interval};
use ts_rs::TS;

pub fn hash_url(this: &(impl Display + ?Sized)) -> String {
    let hash = Sha256::digest(this.to_string().as_bytes());
    BASE32.encode(&hash)[..7].to_string()
}

async fn update_wallpaper_internal(app_handle: AppHandle) -> Result<()> {
    let config = app_handle.get_config().await;
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
            where source = $1 and was_set = 0
            order by date desc",
            source_str
        )
        .fetch_optional(&mut app_handle_clone.db().await.acquire().await?)
        .await?
        .ok_or(anyhow!("No wallpapers"))
    };

    let wp = get_wp().await;
    let app_handle_clone = app_handle.clone();

    let wallpaper = match wp {
        Ok(wallpaper) => Ok::<Wallpaper, anyhow::Error>(wallpaper),
        Err(e) => {
            let e = e as anyhow::Error;
            if e.to_string().contains("No wallpapers") {
                match source {
                    Source::Subreddit(sub) => get_from_subreddit(sub, &app_handle_clone).await?,
                }
            } else {
                return Err(e);
            };
            let wallpaper = get_wp().await?;
            Ok(wallpaper)
        }
    }?;
    trim_queue(&app_handle).await?;
    let cache_dir = &app_handle.get_config().await.cache_dir;
    let cache_dir_clone = cache_dir.clone();
    let downloaded_file = fs::read_dir(cache_dir)?.find_map(|e| {
        if let Some(name) = (e.as_ref().ok()?.file_name().into_string()).ok() &&
             hash_url(&name) == wallpaper.id
        {Some(name)}
        else {None}
    });

    let wallpaper_path = if let Some(path) = downloaded_file {
        Ok(cache_dir_clone.join(path))
    } else {
        download_wallpaper(&app_handle, &wallpaper.data_url)
            .await
            .map(|p| cache_dir_clone.join(p))
    }?;

    wallpaper::set_from_path(
        wallpaper_path
            .to_str()
            .ok_or("Invalid path")
            .map_err(|e| anyhow!("{e:#?}"))?,
    )
    .map_err(|e| anyhow!(e.to_string()))?;

    let now = chrono::Utc::now().naive_utc();
    query!(
        "---sql
        update queue set was_set = 1, date = $1 where id = $2",
        now,
        wallpaper.id
    )
    .execute(&mut app_handle.db().await.acquire().await?)
    .await?;

    app_handle.emit_all("wallpaper_updated", &wallpaper)?;

    app_handle
        .tray_handle()
        .get_item("open_info")
        .set_title(wallpaper.name.as_str())?;
    eprintln!("New wallpaper: {:#?}", wallpaper);
    Ok(())
}

pub fn setup_changer(app_handle: AppHandle) -> Sender<Duration> {
    let (tx_dur, mut rx_dur) = async_runtime::channel::<Duration>(100);
    let mut handle: Option<JoinHandle<_>> = None;
    async_runtime::spawn((async move || loop {
        let dur = if let Some(dur) = rx_dur.recv().await {
            dur
        } else {
            break;
        };
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

pub async fn download_wallpaper(app_handle: &AppHandle, wp_url: impl Display) -> Result<PathBuf> {
    let wp_res = reqwest::get(&wp_url.to_string()).await?;
    let wallpaper_filename = format!(
        "{}.{}",
        hash_url(&wp_url),
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
    let config = app_handle.get_config().await;
    let cache_folder = config.cache_dir.clone();
    let wallpaper_filename = cache_folder.join(wallpaper_filename);
    // If the folder isn't full of photos
    while read_dir_sync(&cache_folder)?
        .filter_map(|f| Some(f.ok()?.metadata().ok()?.len()))
        .sum::<u64>()
        + wp_res.content_length().unwrap_or(0)
        >= (config.cache_size * 1024.0 * 1024.0).floor() as u64
    {
        let oldest_download = read_dir_sync(&config.cache_dir)?
            .find_map(|f| {
                let f = f.ok()?;
                if !mime_guess::from_path(f.path())
                    .iter()
                    .any(|g| g.type_() == IMAGE)
                {
                    return None;
                }
                Some(f.path())
            })
            .ok_or(anyhow!("No downloads to delete"))?;
        fs::remove_file(oldest_download)?;
    }
    fs::write(&wallpaper_filename, wp_res.bytes().await?)?;
    Ok(wallpaper_filename)
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

#[tauri::command]
pub async fn set_wallpaper(app_handle: AppHandle, wallpaper: Wallpaper) -> Result<(), String> {
    (async move || -> Result<()> {
        let wallpaper = wallpaper;
        let cache_dir = &app_handle.get_config().await.cache_dir;
        let cache_dir_clone = cache_dir.clone();
        let downloaded_file = fs::read_dir(cache_dir)?.find_map(|e| {
            if let Some(name) = (e.as_ref().ok()?.file_name().into_string()).ok() &&
                     hash_url(&name) == wallpaper.id
                {Some(name)}
                else {None}
        });

        let wallpaper_path = if let Some(path) = downloaded_file {
            Ok(cache_dir_clone.join(path))
        } else {
            download_wallpaper(&app_handle, &wallpaper.data_url)
                .await
                .map(|p| cache_dir_clone.join(p))
        }?;

        wallpaper::set_from_path(
            wallpaper_path
                .to_str()
                .ok_or("Invalid path")
                .map_err(|e| anyhow!("{e:#?}"))?,
        )
        .map_err(|e| anyhow!(e.to_string()))?;

        let now = chrono::Utc::now().naive_utc();
        query!(
            "---sql
            update queue set was_set = 1, date = $1 where id = $2",
            now,
            wallpaper.id
        )
        .execute(&mut app_handle.db().await.acquire().await?)
        .await?;

        app_handle
            .tray_handle()
            .get_item("open_info")
            .set_title(wallpaper.name.as_str())?;
        app_handle.emit_all("wallpaper_updated", wallpaper.clone())?;
        eprintln!("New wallpaper: {:#?}", wallpaper);
        Ok(())
    })()
    .await
    .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn get_wallpaper(app_handle: AppHandle, wallpaper: Wallpaper) -> Result<String, String> {
    (async move || -> Result<String> {
        let cache_dir = &app_handle.get_config().await.cache_dir;
        let file = read_dir_sync(cache_dir)?.find_map(|e| {
            if let Some(name) = (e.as_ref().ok()?.file_name().into_string()).ok() && 
            name != "" && name.starts_with(&wallpaper.id)
                {Some(e.as_ref().ok()?.path())}
                else {None}
        });
        let data = if let Some(file) = file {
            read(file).await?
        } else {
            let file = download_wallpaper(&app_handle, &wallpaper.data_url).await?;
            read(file).await?
        };
        // base64 encode data
        Ok(general_purpose::STANDARD.encode(&data))
    })()
    .await
    .map_err(|e| e.to_string())
}
