use crate::app_config::AppHandleExt;
use crate::sources::reddit::get_from_subreddit;
use crate::wallpaper_changer::download_wallpaper;
use anyhow::{anyhow, Result};
use reddw_shared::{Wallpaper, Source};
use sqlx::migrate::MigrateDatabase;
use sqlx::{migrate, query, query_as, Pool, Sqlite};
use std::fs::{self, read_dir};
use tauri::async_runtime::spawn;
use tauri::Manager;

pub type DB = Pool<Sqlite>;

pub async fn manage_queue(app: tauri::AppHandle) -> Result<()> {
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
    app.manage(db);
    migrate!().run(app.state::<DB>().inner()).await?;
    Ok(())
}

#[tauri::command]
pub async fn get_queue(app: tauri::AppHandle) -> Result<Vec<Wallpaper>, String> {
    let db = app.state::<DB>();
    let queue = query_as!(
        Wallpaper,
        "SELECT * FROM queue where was_set = 0 ORDER BY date DESC"
    )
    .fetch_all(&mut db.acquire().await.map_err(|e| e.to_string())?)
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
    .execute(&mut app.state::<DB>().acquire().await?)
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn cache_queue(app: tauri::AppHandle) -> Result<usize, String> {
    let config = app.get_config().await.clone();
    let mut ret = usize::default();
    trim_queue(&app).await.map_err(|e| e.to_string())?;
    for source in config.sources.iter() {
        match source {
            Source::Subreddit(sub) => {
                ret += get_from_subreddit(sub, &app)
                    .await
                    .map_err(|e| e.to_string())?
            }
        }
    }
    spawn(download_queue(app));
    Ok(ret)
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
