use crate::{
    app_handle_ext::AppHandleExt,
    log::{LogLevel, LogBehaviours},
    // queue::manage_queue,
    source_host::{PluginHostMode, SourcePlugins},
    watcher::watch_path_sync,
};
use anyhow::{anyhow, Result};
use futures::{StreamExt, TryFutureExt};
use macros::command;
use notify::RecursiveMode;
use reddw_source_plugin::GenericValue;
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
    async_runtime::{spawn, Mutex, Sender},
    AppHandle, Manager,
};
use ts_rs::TS;

const CONFIG_SYNC_DURATION: Duration = Duration::from_millis(25);

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
    pub logging: LogBehaviours,
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
            logging: LogBehaviours::new(),
        }
    }
}

pub fn build(app: AppHandle, tx_interval: Sender<Duration>) -> tauri::Result<()> {
    let config_dir = app.path_resolver().app_config_dir().unwrap();
    if !&config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }
    let config_path = app.get_config_path();
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
        app.manage(tx_interval);
        app.manage(Mutex::new(config.clone()));
    }

    let mut watch = watch_path_sync(
        app.app_handle(),
        &config_path,
        RecursiveMode::NonRecursive,
        CONFIG_SYNC_DURATION,
    )
    .map_err(|err| std::io::Error::other(err))?;
    spawn(async move {
        let app_clone = app.app_handle();
        while let Some(_) = watch.next().await {
            let app = app.app_handle();
            let config_path = app.get_config_path();
            (async move || {
                let mut config = serde_json::from_str::<AppConfig>(&read_to_string(&config_path)?)?;
                let old_config = app.get_config().await;
                let removed = old_config
                    .sources
                    .iter()
                    .filter(|(source, _)| !config.sources.contains_key(source.to_owned()))
                    .map(|(s, v)| (s.to_owned(), v.to_owned()))
                    .collect::<Vec<_>>();
                let added = config
                    .sources
                    .iter()
                    .filter(|(source, updated_source_config)| {
                        if let Some(old_source) = old_config.sources.get(source.to_owned()) {
                            old_source != updated_source_config.to_owned()
                        } else {
                            true
                        }
                    })
                    .map(|(s, v)| (s.to_owned(), v.to_owned()))
                    .collect::<Vec<_>>();

                for (source, _) in removed {
                    config = update_config(
                        app.app_handle(),
                        ConfigUpdate::RemoveSource(source.to_owned()),
                    )
                    .await?;
                }

                for (source, data) in added {
                    config = update_config(
                        app.app_handle(),
                        ConfigUpdate::AddSource(
                            source.to_owned(),
                            GenericValue(serde_cbor::from_slice(&serde_json::to_vec(
                                &data.to_owned(),
                            )?)?),
                        ),
                    )
                    .await?;
                }

                if old_config.interval != config.interval {
                    config = update_config(
                        app.app_handle(),
                        ConfigUpdate::ChangeInterval {
                            interval: config.interval,
                        },
                    )
                    .await?;
                }
                app.emit_all("config_changed", &config)?;
                *app.state::<Mutex<AppConfig>>().lock().await = config;
                Ok(())
            })()
            .unwrap_or_else(|err: anyhow::Error| app_clone.log(&err, LogLevel::Error))
            .await;
        }
    });
    Ok(())
}

#[command]
pub async fn get_config(app: AppHandle) -> AppConfig {
    app.get_config().await
}

pub async fn update_config(app: AppHandle, update: ConfigUpdate) -> Result<AppConfig> {
    let current_config = app.get_config().await;
    let updated_config = match update {
        ConfigUpdate::Other(new_config) => AppConfig {
            sources: current_config.sources,
            interval: current_config.interval,
            ..new_config
        },
        ConfigUpdate::AddSource(plugin_instance, params) => {
            let (plugin, instance) = plugin_instance
                .split_once('_')
                .ok_or(anyhow!("Invalid source"))?;
            let overrode = {
                let sources = app.state::<SourcePlugins>();
                let mut sources = sources.lock().await;
                let source = sources.get_mut(plugin).ok_or(anyhow!("Invalid source"))?;
                source
                    .register_instance(instance.to_string(), params.clone())
                    .await
                    .map_err(|err| anyhow!("{err:#?}"))?
            };
            if overrode {
                query!("delete from queue where source = ?", plugin_instance)
                    .execute(&app.db().await)
                    .await?;
            }
            let mut sources = current_config.sources;
            sources.insert(
                plugin_instance,
                serde_json::from_slice(&serde_cbor::to_vec(&params.0)?)?,
            );
            AppConfig {
                sources,
                ..current_config
            }
        }
        ConfigUpdate::RemoveSource(plugin_instance) => {
            let (plugin, instance) = plugin_instance
                .split_once('_')
                .ok_or(anyhow!("Invalid source"))?;
            let sources = app.state::<SourcePlugins>();
            let mut sources = sources.lock().await;
            let source = sources.get_mut(plugin).ok_or(anyhow!("Invalid source"))?;
            source
                .deregister_instance(instance.to_string())
                .await
                .map_err(|err| anyhow!("{err:#?}"))?;
            query!("delete from queue where source = ?", plugin_instance)
                .execute(&app.db().await)
                .await?;
            let mut sources = current_config.sources;
            sources.remove(&plugin_instance);
            AppConfig {
                sources,
                ..current_config
            }
        }
        ConfigUpdate::ChangeInterval { interval } => {
            let tx_interval = app.state::<Sender<Duration>>();
            tx_interval.send(interval).await?;
            AppConfig {
                interval,
                ..current_config
            }
        }
    };
    Ok(updated_config)
}

pub mod update_command {
    use crate::{app_handle_ext::AppHandleExt, watcher::FileWatches};
    use anyhow::{anyhow, Result};
    use macros::command;
    use notify::{RecursiveMode, Watcher};
    use tauri::{AppHandle, Manager};

    #[command]
    pub async fn update_config(app: AppHandle, update: super::ConfigUpdate) -> Result<()> {
        let updated = super::update_config(app.app_handle(), update)
            .await
            .map_err(|err| anyhow!("{err}"))?;
        let watches = app.state::<FileWatches>();
        let mut watches = watches.lock().await;
        let config_path = app.get_config_path();
        let mut watcher = watches.get_mut(&config_path);
        if let Some(watcher) = &mut watcher {
            watcher.unwatch(&config_path)?;
        }
        app.emit_all("config_changed", &updated)?;
        tokio::fs::write(&config_path, serde_json::to_vec_pretty(&updated)?).await?;
        if let Some(watcher) = watcher {
            watcher.watch(&config_path, RecursiveMode::NonRecursive)?;
        }
        Ok(())
    }
}

#[command]
pub async fn select_folder() -> Result<PathBuf> {
    let folder = rfd::AsyncFileDialog::new()
        .pick_folder()
        .await
        .ok_or(anyhow!("No folder picked"))?;
    Ok(folder.path().to_path_buf())
}

#[derive(TS, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum ConfigUpdate {
    AddSource(String, GenericValue),
    RemoveSource(String),
    ChangeInterval {
        #[ts(type = "{secs: number, nanos: number}")]
        interval: Duration,
    },
    /// Config changes which do not require special behaviour on the backend
    Other(AppConfig),
}
