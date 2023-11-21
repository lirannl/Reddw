use anyhow::{anyhow, Result};
use notify::{recommended_watcher, RecursiveMode, Watcher};
use reddw_source_plugin::SourceParameters;
use rfd;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, read_to_string},
    path::{Path, PathBuf},
    thread::{self, spawn},
    time::Duration,
};
use tauri::{
    async_runtime::{block_on, Mutex, Sender},
    Manager,
};
use ts_rs::TS;

use crate::{app_handle_ext::AppHandleExt, queue::manage_queue, source_host::PluginHostMode};

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export)]
pub struct AppConfig {
    /// Allow fetching wallpapers from Not Safe For Work sources (aka - sexually explicit content/gore)
    pub allow_nsfw: bool,
    pub display_background: bool,
    pub sources: HashMap<String, SourceParameters>,
    #[ts(type = "{secs: number, nanos: number}")]
    /// How often to switch new wallpapers (in seconds)
    pub interval: Duration,
    pub cache_dir: PathBuf,
    pub plugin_host_mode: PluginHostMode,
    // Max cache size, in megabytes
    pub cache_size: f64,
    pub plugins_dir: Option<PathBuf>,
    pub history_amount: i32,
    pub theme: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            allow_nsfw: false,
            sources: HashMap::new(),
            interval: Duration::from_secs(60 * 60),
            cache_dir: PathBuf::new(),
            cache_size: 100.0,
            history_amount: 10,
            plugins_dir: None,
            plugin_host_mode: PluginHostMode::Daemon,
            theme: "default".to_string(),
            display_background: true,
        }
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
            let def_conf_str = serde_json::to_string_pretty(&def_conf)
                .expect("Failed to serialize default config");
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
            if res.is_ok() {
                (|| -> Result<()> {
                    let config =
                        serde_json::from_str::<AppConfig>(&read_to_string(&config_path_clone)?)?;
                    let old_config = block_on(app.state::<Mutex<AppConfig>>().lock()).clone();
                    if old_config.interval != config.interval {
                        tx_interval
                            .try_send(config.interval)
                            .or_else(|e| Err(anyhow!("{:#?}", e)))?;
                    }
                    *block_on(app.state::<Mutex<AppConfig>>().lock()) = config.clone();
                    if old_config.cache_dir != config.cache_dir {
                        block_on(manage_queue(&app))?;
                    }
                    app.emit_all("config_changed", vec![old_config, config])?;
                    Ok(())
                })()
                .unwrap_or_else(|err| {
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
