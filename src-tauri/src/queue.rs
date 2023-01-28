use std::fs;
use anyhow::{anyhow, Result};
use sqlx::{migrate, query_as, Pool, Sqlite};
use tauri::Manager;

use crate::{wallpaper_changer::Wallpaper, app_config::AppConfig};

pub trait AppDb {
    async fn update_queue(self, queue: &Vec<Wallpaper>) -> Result<()>;
}

pub type DB = Pool<Sqlite>;

pub async fn manage_queue(app: tauri::AppHandle) -> Result<()> {
    let cache_dir = &app.state::<AppConfig>().cache_dir;
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)?;
    }
    let db: DB = {
        sqlx::SqlitePool::connect(
            cache_dir
                .join("queue.db")
                .to_str()
                .ok_or(anyhow!("No path"))?,
        )
        .await?
    };
    app.manage(db);
    migrate!().run(app.state::<DB>().inner()).await?;
    Ok(())
}

#[tauri::command]
pub async fn get_queue(app: tauri::AppHandle) -> Result<Vec<Wallpaper>> {
    let db = app.state::<DB>();
    let queue = query_as!(
        Wallpaper,
        "SELECT * FROM queue where was_set = 0 ORDER BY date DESC"
    )
    .fetch_all(db.inner())
    .await?;
    Ok(queue)
}
