use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use notify::{
    event::{EventKind, ModifyKind},
    recommended_watcher, RecursiveMode, Watcher,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, read_to_string},
    path::{Path, PathBuf},
    thread::{self, spawn},
    time::Duration,
};
use tauri::{
    async_runtime::{block_on, Mutex, Sender},
    AppHandle, Manager,
};
use ts_rs::TS;

use crate::wallpaper_changer::Wallpaper;

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export)]
pub enum Source {
    Subreddit(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export)]
pub struct AppConfig {
    /// Allow fetching wallpapers from Not Safe For Work sources (aka - sexually explicit content/gore)
    pub allow_nsfw: bool,
    pub sources: Vec<Source>,
    #[ts(type = "{secs: number, nanos: number}")]
    /// How often to switch new wallpapers (in seconds)
    pub interval: Duration,
    #[ts(skip)]
    pub history: Vec<Wallpaper>,
    pub cache_dir: PathBuf,
    // Max cache size, in megabytes
    pub cache_size: f64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            allow_nsfw: false,
            sources: vec![Source::Subreddit("wallpapers".to_string())],
            interval: Duration::from_secs(60 * 60),
            history: vec![],
            cache_dir: PathBuf::new(),
            cache_size: 100.0,
        }
    }
}

pub trait AppHandleExt {
    fn get_config_path(&self) -> PathBuf;
}

impl AppHandleExt for AppHandle {
    fn get_config_path(&self) -> PathBuf {
        let config_dir = self.path_resolver().app_config_dir().unwrap();
        Path::join(&config_dir, "config.json")
    }
}

lazy_static! {
    pub static ref CONFIG: Mutex<AppConfig> = Mutex::new(AppConfig::default());
    pub static ref CONFIG_PATH: Mutex<PathBuf> = Mutex::new(PathBuf::new());
}

pub async fn build(app: tauri::AppHandle, tx_interval: Sender<Duration>) -> tauri::Result<()> {
    let config_dir = app.path_resolver().app_config_dir().unwrap();
    if !&config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }
    let config_path = Path::join(&config_dir, "config.json");
    *CONFIG_PATH.lock().await = config_path.clone();
    let config_path_clone = config_path.clone();
    if !config_path.exists() {
        let mut config = AppConfig::default();
        config.cache_dir = app.path_resolver().app_cache_dir().unwrap();
        let config_json = serde_json::to_string_pretty(&config)?;
        fs::write(&config_path, config_json)?;
    }
    {
        let config_json = read_to_string(&config_path).unwrap();
        if config_json == "" {
            return Ok(());
        }
        let config: AppConfig = serde_json::from_str(&config_json).unwrap_or_else(|e| {
            println!("Failed to parse config: {:#?}", e);
            let mut def_conf = AppConfig::default();
            def_conf.cache_dir = app.path_resolver().app_cache_dir().unwrap();
            def_conf
        });
        tx_interval
            .try_send(config.interval)
            .or(Err(tauri::Error::FailedToSendMessage))?;
        *CONFIG.lock().await = config;
    }
    spawn(move || {
        let mut watcher = recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res && EventKind::Modify(ModifyKind::Any) == event.kind {
                    (|| -> Result<()> {
                        let config = serde_json::from_str::<AppConfig>(&read_to_string(
                            &config_path_clone)?)
                        //     .map(|valid_conf: AppConfig| {
                        //     println!("Emitting config_changed");
                        //     app.get_window("main").map(|w| w.emit("config_changed", valid_conf.clone()));
                        //     valid_conf
                        // })
                        ?;
                        let old_config = block_on(CONFIG.lock()).clone();
                        if old_config.interval != config.interval {
                            tx_interval
                                .try_send(config.interval)
                                .or_else(|e| Err(anyhow!("{:#?}", e)))?;
                        }
                        *block_on(CONFIG.lock()) = config;
                        Ok(())
                    })().unwrap_or_else(|err| {
                        eprintln!("{:#?}", err);
                    });
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

#[tauri::command]
pub async fn set_config(app: tauri::AppHandle, app_config: AppConfig) -> tauri::Result<()> {
    let config_json = serde_json::to_string_pretty(&app_config)?;
    fs::write(app.get_config_path(), config_json)?;
    Ok(())
}
