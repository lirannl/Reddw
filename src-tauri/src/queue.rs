use crate::app_handle_ext::AppHandleExt;
use crate::source_host::Plugins;
use crate::wallpaper_changer::download_wallpaper;
use ::futures::future::join_all;
use anyhow::{anyhow, Result};
use reddw_source_plugin::Wallpaper;
use sqlx::migrate::MigrateDatabase;
use sqlx::{migrate, query, query_as, Pool, Sqlite};
use std::fs::{self, read_dir};
use tauri::async_runtime::{spawn, Mutex};
use tauri::{AppHandle, Manager};

pub type DB = Pool<Sqlite>;

pub async fn manage_queue(app: &AppHandle) -> Result<()> {
    let cache_dir = app.get_config().await.cache_dir;
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)?;
    }
    let db_path = &cache_dir.join("queue.db");
    let db_url = db_path.to_str().ok_or(anyhow!("Invalid path"))?;
    if !db_path.exists() {
        Sqlite::create_database(db_url).await?;
    }
    let db: DB = { sqlx::SqlitePool::connect(db_url).await? };
    migrate!().run(&db).await?;
    app.manage(Mutex::new(db));
    Ok(())
}

#[tauri::command]
pub async fn get_queue(app: tauri::AppHandle) -> Result<Vec<Wallpaper>, String> {
    let db = app.db().await;
    let queue = query_as!(Wallpaper, "SELECT * FROM queue ORDER BY date DESC")
        .fetch_all(&db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(queue)
}

pub async fn trim_queue(app: &tauri::AppHandle) -> Result<()> {
    let config = app.get_config().await.clone();
    // Remove history entries except for the newest x
    query!(
        "---sql
        delete from queue 
        where id in 
        (select id from queue 
            where was_set = 1 
            order by date
            limit -1 offset ?)
        ;
        ",
        config.history_amount
    )
    .execute(&app.db().await)
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn cache_queue(app: tauri::AppHandle) -> Result<usize, String> {
    async {
        trim_queue(&app).await?;
        let plugins = app.state::<Plugins>();
        let mut plugins = plugins.lock().await;
        let wallpapers = join_all(plugins.values_mut().map(|plugin| {
            let app = app.app_handle();
            async move {
                let config = app.get_config().await;
                let name = plugin.name.clone();
                let mut wallpapers = Vec::new();
                for instance in config.sources.keys().filter_map(|key| {
                    let (plugin, instance) = key.split_once("_")?;
                    if plugin != name {
                        return None;
                    }
                    Some(instance)
                }) {
                    wallpapers.extend(plugin.get_wallpapers(instance.to_string()).await?);
                }
                Ok(wallpapers)
            }
        }))
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?;
        let wallpapers = wallpapers.into_iter().flat_map(|w| w).collect::<Vec<_>>();
        spawn(download_queue(app.app_handle()));
        Ok(wallpapers.len())
    }
    .await
    .map_err(|err: anyhow::Error| err.to_string())
}

pub async fn download_queue(app: tauri::AppHandle) -> Result<()> {
    let queue = get_queue(app.app_handle()).await.map_err(|e| anyhow!(e))?;
    for wallpaper in queue {
        let app_clone = app.app_handle();

        let config = app_clone.get_config().await;
        // If the folder isn't full of photos
        if read_dir(config.cache_dir)?
            .filter_map(|f| Some(f.ok()?.metadata().ok()?.len()))
            .sum::<u64>()
            < (config.cache_size * 1024.0 * 1024.0).round() as u64
        {
            download_wallpaper(&app_clone, wallpaper.data_url).await?;
        }
    }
    Result::<()>::Ok(())
}
