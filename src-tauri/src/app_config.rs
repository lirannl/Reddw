use anyhow::{anyhow, Result};
use notify::{
    event::{EventKind, ModifyKind},
    recommended_watcher, RecursiveMode, Watcher,
};
use rfd;
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

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export)]
pub enum Source {
    Subreddit(String),
}
impl Default for Source {
    fn default() -> Self {
        Self::Subreddit("wallpapers".to_string())
    }
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
    pub cache_dir: PathBuf,
    // Max cache size, in megabytes
    pub cache_size: f64,
    pub history_amount: i32,
    pub theme: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            allow_nsfw: false,
            sources: vec![Default::default()],
            interval: Duration::from_secs(60 * 60),
            cache_dir: PathBuf::new(),
            cache_size: 100.0,
            history_amount: 10,
            theme: "default".to_string(),
        }
    }
}

pub trait AppHandleExt {
    fn get_config_path(&self) -> PathBuf;
    async fn get_config(&self) -> AppConfig;
}

impl AppHandleExt for AppHandle {
    fn get_config_path(&self) -> PathBuf {
        let config_dir = self.path_resolver().app_config_dir().unwrap();
        Path::join(&config_dir, "config.json")
    }
    async fn get_config(&self) -> AppConfig {
        self.state::<Mutex<AppConfig>>().lock().await.clone()
    }
}

pub fn build(app: tauri::AppHandle, tx_interval: Sender<Duration>) -> tauri::Result<()> {
    let config_dir = app.path_resolver().app_config_dir().unwrap();
    if !&config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }
    let config_path = Path::join(&config_dir, "config.json");
    let config_path_clone = config_path.clone();
    if !config_path.exists() {
        let mut config = AppConfig::default();
        config.cache_dir = app
            .path_resolver()
            .app_cache_dir()
            .unwrap_or(Path::new("").to_path_buf());
        let config_json = serde_json::to_string_pretty(&config)?;
        fs::write(&config_path, config_json)?;
    }
    {
        let config_json = read_to_string(&config_path).unwrap();
        let config: AppConfig = serde_json::from_str(&config_json).unwrap_or_else(|e| {
            println!("Failed to parse config: {:#?}", e);
            let mut def_conf = AppConfig::default();
            def_conf.cache_dir = app.path_resolver().app_cache_dir().unwrap();
            let def_conf_str = serde_json::to_string_pretty(&def_conf).expect("Failed to serialize default config");
            fs::write(&config_path, def_conf_str).expect("Failed to write default config");
            def_conf
        });
        tx_interval
            .try_send(config.interval)
            .or(Err(tauri::Error::FailedToSendMessage))?;
        app.manage(Mutex::new(config.clone()));
    }
    spawn(move || {
        let mut watcher = recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res && EventKind::Modify(ModifyKind::Any) == event.kind {
                    (|| -> Result<()> {
                        let config = serde_json::from_str::<AppConfig>(&read_to_string(
                            &config_path_clone)?)?;
                        let old_config = block_on(app.state::<Mutex<AppConfig>>().lock()).clone();
                        if old_config.interval != config.interval {
                            tx_interval
                                .try_send(config.interval)
                                .or_else(|e| Err(anyhow!("{:#?}", e)))?;
                        }
                        app.emit_all("config_changed", Some(config.clone()))?;
                        *block_on(app.state::<Mutex<AppConfig>>().lock()) = config;
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
pub async fn get_config(app: tauri::AppHandle) -> tauri::Result<AppConfig> {
    Ok(app.get_config().await)
}

#[tauri::command]
pub async fn set_config(app: tauri::AppHandle, app_config: AppConfig) -> tauri::Result<()> {
    let config_json = serde_json::to_string_pretty(&app_config)?;
    fs::write(app.get_config_path(), config_json)?;
    Ok(())
}

#[tauri::command]
pub async fn select_folder() -> Result<String, String> {
    let folder = rfd::AsyncFileDialog::new()
        .pick_folder()
        .await
        .ok_or("No folder picked")?;
    Ok(folder.path().to_str().ok_or("Invalid path")?.to_string())
}
