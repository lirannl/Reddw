use lazy_static::lazy_static;
use notify::{
    event::{DataChange, EventKind, ModifyKind},
    recommended_watcher, RecursiveMode, Watcher,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::read_to_string,
    path::Path,
    thread::{self, spawn},
};
use tauri::async_runtime::{block_on, Mutex};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Source {
    Subreddit(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    /// Allow fetching wallpapers from Not Safe For Work sources (aka - sexually explicit content/gore)
    allow_nsfw: bool,
    sources: Vec<Source>,
    /// How often to switch new wallpapers (in seconds)
    interval: f64,
    /// How many wallpapers to keep in the cache
    max_buffer: i64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            allow_nsfw: false,
            sources: vec![Source::Subreddit("wallpapers".to_string())],
            interval: 60.0 * 60.0 * 24.0,
            max_buffer: 1000 * 1000 * 100,
        }
    }
}

lazy_static! {
    pub static ref CONFIG: Mutex<AppConfig> = Mutex::new(AppConfig::default());
}

pub async fn build(app: tauri::AppHandle) -> tauri::Result<()> {
    let config_dir = app.path_resolver().app_config_dir().unwrap();
    if !&config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?;
    }
    let config_path = Path::join(&config_dir, "config.json");
    let config_path_clone = config_path.clone();
    if !config_path.exists() {
        let config = AppConfig::default();
        let config_json = serde_json::to_string_pretty(&config)?;
        std::fs::write(&config_path, config_json)?;
    }
    {
        let config_json = read_to_string(&config_path).unwrap();
        let config: AppConfig = serde_json::from_str(&config_json).unwrap();
        *CONFIG.lock().await = config;
    }
    spawn(move || {
        let mut watcher =
            recommended_watcher(move |res: notify::Result<notify::Event>| match res {
                Ok(event) => {
                    println!("event: {:?}", event);
                    let config_json = read_to_string(&config_path_clone).unwrap();
                    let config: AppConfig = serde_json::from_str(&config_json).unwrap();
                    *block_on(CONFIG.lock()) = config;
                }
                Err(e) => {
                    println!("watch error: {:?}", e);
                }
            })
            .unwrap();

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher
            .watch(&config_path.as_path(), RecursiveMode::NonRecursive)
            .unwrap();
        loop {
            thread::sleep(std::time::Duration::from_secs(1));
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn get_config() -> tauri::Result<AppConfig> {
    Ok((*CONFIG.lock().await).clone())
}
