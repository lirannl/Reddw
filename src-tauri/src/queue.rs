use std::fs;

use anyhow::{anyhow, Result};
use notify::{recommended_watcher, Watcher};
use sqlx::{query, Executor};
use tauri::{async_runtime::Mutex, Manager, api::path::cache_dir};

use crate::wallpaper_changer::Wallpaper;

pub type Queue = Mutex<Vec<Wallpaper>>;

pub trait AppDb {
    async fn update_queue(self, queue: &Queue) -> Result<()>;
}

pub type DB = sqlx::SqlitePool;

pub async fn manage_history(app: tauri::AppHandle) -> Result<()> {
    let cache_dir = cache_dir()
        .ok_or(anyhow!("No cache dir"))?;
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)?;
    }
    let db: DB = {
        let this = app.app_handle();
        sqlx::SqlitePool::connect(
            cache_dir
                .join("queue.db")
                .to_str()
                .ok_or(anyhow!("No path"))?,
        )
        .await?
    };
    app.manage(db);
    app.state::<DB>()
        .execute(query!(
            "
        CREATE TABLE IF NOT EXISTS queue (
            id	TEXT,
            file_name	TEXT NOT NULL,
            date	TEXT NOT NULL,
            info_url	TEXT NOT NULL,
            source TEXT NOT NULL,
            was_set 	NUMERIC NOT NULL DEFAULT 0,
            PRIMARY KEY(id)
        );
    "
        ))
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn get_queue(app: tauri::AppHandle) -> Vec<Wallpaper> {
    app.state::<Queue>().lock().await.clone()
}

pub async fn watch_queue(app: tauri::AppHandle) -> Result<()> {
    let mut watcher = recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            if let notify::EventKind::Modify(_) = event.kind {}
        }
    })?;
    watcher
        .watch(
            &cache_dir().unwrap(),
            notify::RecursiveMode::NonRecursive,
        )
        .unwrap();
    Ok(())
}
