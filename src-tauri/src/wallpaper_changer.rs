use crate::app_config::{AppHandleExt, Source};
use crate::queue::DB;
use crate::sources::reddit::get_from_subreddit;
use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use data_encoding::BASE32;
use mime_guess::mime::IMAGE;
use mime_guess::Mime;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{query, query_as, FromRow};
use std::fmt::Display;
use std::fs::{self, read_dir};
use std::str::FromStr;
use std::time::Duration;
use tauri::{
    async_runtime::{self, JoinHandle, Sender},
    AppHandle, Manager,
};
use tokio::time::interval;
use ts_rs::TS;

pub fn hash_url(this: &(impl Display + ?Sized)) -> String {
    let hash = Sha256::digest(this.to_string().as_bytes());
    BASE32.encode(&hash)[..7].to_string()
}

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
    #[serde(skip)]
    pub was_set: bool,
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

    query!(
        "---sql
        update queue set was_set = 1 where id = $1",
        wallpaper.id
    )
    .execute(&mut app_handle.state::<DB>().acquire().await?)
    .await?;

    app_handle
        .tray_handle()
        .get_item("update_wallpaper")
        .set_title(wallpaper.name.as_str())?;
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

pub async fn download_wallpaper(app_handle: &AppHandle, wp_url: impl Display) -> Result<String> {
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
    // If the folder isn't full of photos
    while read_dir(&cache_folder)?
        .filter_map(|f| Some(f.ok()?.metadata().ok()?.len()))
        .sum::<u64>()
        + wp_res.content_length().unwrap_or(0)
        >= (config.cache_size * 1024.0 * 1024.0).floor() as _
    {
        let oldest_download = read_dir(&config.cache_dir)?
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
    fs::write(
        &cache_folder.join(&wallpaper_filename),
        wp_res.bytes().await?,
    )?;
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
