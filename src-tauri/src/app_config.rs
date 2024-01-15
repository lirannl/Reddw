use anyhow::{anyhow, Result};
use macros::command;
use notify::RecursiveMode;
use rfd;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::query;
use std::{
    collections::HashMap,
    fmt::Debug,
    fs::{self, read_to_string},
    path::{Path, PathBuf},
    time::Duration,
};
use tauri::{
    async_runtime::{Mutex, Sender},
    AppHandle, Manager,
};
use ts_rs::TS;

use crate::{
    app_handle_ext::AppHandleExt,
    log::LogLevel,
    // queue::manage_queue,
    source_host::PluginHostMode,
    watcher::watch_path,
};

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export)]
pub struct AppConfig {
    pub display_background: bool,
    #[ts(type = "Record<string, any>")]
    pub sources: HashMap<String, Value>,
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

pub fn build(app: AppHandle, tx_interval: Sender<Duration>) -> tauri::Result<()> {
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
            app.log(
                &format!("Failed to parse config: {:#?}", e),
                LogLevel::Error,
            );
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

    let _handle = watch_path(
        &config_path,
        move |_, app| {
            let tx_interval = tx_interval.clone();
            let config_path = config_path_clone.clone();
            Box::pin(async move {
                let tx_interval = &tx_interval;
                let config = serde_json::from_str::<AppConfig>(&read_to_string(&config_path)?)?;
                let old_config = app.state::<Mutex<AppConfig>>().lock().await.clone();
                if old_config.interval != config.interval {
                    tx_interval
                        .try_send(config.interval)
                        .or_else(|e| Err(anyhow!("{:#?}", e)))?;
                }
                *app.state::<Mutex<AppConfig>>().lock().await = config.clone();
                // if old_config.cache_dir != config.cache_dir {
                //     manage_queue(&app).await.map_err(|err| anyhow!("{err:#?}"))?;
                // }
                app.emit_all("config_changed", vec![old_config, config])?;

                Ok(())
            })
        },
        app,
        RecursiveMode::NonRecursive,
    )
    .map_err(|err| std::io::Error::other(err))?;
    Ok(())
}

#[command]
pub async fn get_config(app: AppHandle) -> AppConfig {
    app.get_config().await
}

#[command]
pub async fn set_config(app: AppHandle, app_config: AppConfig) -> Result<()> {
    let sources_old = app.get_config().await.sources;
    let sources_new = &app_config.sources;
    let (_added, removed) = (
        sources_new
            .iter()
            .filter(|(k, _)| !sources_old.contains_key(k.to_owned())),
        sources_old
            .iter()
            .filter(|(k, _)| !sources_new.contains_key(k.to_owned())),
    );
    for (source, _data) in removed {
        if let Err(err) = query!("delete from queue where source = ?", source)
            .execute(&app.db().await)
            .await
        {
            app.log(&err, LogLevel::Error);
        }
    }

    let config_json = serde_json::to_string_pretty(&app_config)?;
    fs::write(app.get_config_path(), config_json)?;
    Ok(())
}

#[command]
pub async fn select_folder() -> Result<PathBuf> {
    let folder = rfd::AsyncFileDialog::new()
        .pick_folder()
        .await
        .ok_or(anyhow!("No folder picked"))?;
    Ok(folder.path().to_path_buf())
}
