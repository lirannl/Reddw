use anyhow::{anyhow, bail, Result};
use reddw_source_plugin::ReddwSourceHandle;
use serde::{Deserialize, Serialize};
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::{collections::HashMap, fs::read_dir, process::Stdio};
use tauri::{AppHandle, Manager};
use tokio::{fs, process::Command, sync::Mutex};
use ts_rs::TS;

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub enum PluginHostMode {
    Daemon,
    LowRAM,
}

use crate::app_handle_ext::AppHandleExt;

pub type Plugins = Mutex<HashMap<String, ReddwSourceHandle>>;

pub async fn host_plugins(app: AppHandle) -> Result<()> {
    let config = app.get_config().await;
    let plugins_dir = {
        match config.plugins_dir {
            Some(path) => path,
            None => {
                let path = app
                    .path_resolver()
                    .app_config_dir()
                    .expect("App config folder could't be determined")
                    .join("plugins");
                path
            }
        }
    };
    if let Ok(exists) = fs::try_exists(&plugins_dir).await
        && !exists
    {
        fs::create_dir_all(&plugins_dir).await?;
    }
    let mut plugins = HashMap::new();
    let plugin_files = read_dir(plugins_dir)?.filter_map(|f| f.ok());
    for plugin in plugin_files {
        let sources = config.sources.clone();
        let mut plugin_name = (format!("{:?}", plugin.file_name()),);
        let plugin_name_borrow = &mut plugin_name;
        let plugins = &mut plugins;
        (async move || -> Result<()> {
            let plugin_name = plugin_name_borrow;
            // Check execute bit
            #[cfg(target_family = "unix")]
            {
                let permissions: Vec<u8> = format!("{:o}", plugin.metadata()?.permissions().mode())
                    [3..]
                    .chars()
                    .filter_map(|c| format!("{c}").parse().ok())
                    .collect();
                if permissions.iter().all(|p| p % 2 == 0) {
                    bail!("No execute permissions")
                }
            }
            let mut plugin = ReddwSourceHandle::new(
                Command::new(plugin.path())
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    // .stderr(Stdio::piped())
                    .spawn()?,
            )
            .await
            .map_err(|err| anyhow!("Couldn't spawn plugin {} {err:#?}", plugin_name.0))?;
            plugin_name.0 = plugin.name.clone();
            let instances = sources.into_iter().filter_map(|(key, parameters)| {
                if let Some((plugin, instance)) = key.split_once("_")
                    && plugin == plugin_name.0
                {
                    Some((instance.to_string(), parameters))
                } else {
                    None
                }
            });
            for (id, params) in instances {
                plugin
                    .register_instance(id.to_string(), params)
                    .await
                    .map_err(|err| anyhow!("{err:#?}"))?;
            }
            plugins.insert(plugin_name.0.clone(), plugin);
            Ok(())
        })()
        .await
        .unwrap_or_else(|err| {
            app.log(
                &format!("Couldn't load plugin {}: {:#?}", plugin_name.0, err),
                crate::log::LogLevel::Error,
            );
        });
    }
    app.manage::<Plugins>(Mutex::new(plugins));
    Ok(())
}

#[tauri::command]
pub async fn query_available_source_plugins(app: AppHandle) -> Result<Vec<String>, String> {
    let state = app.state::<Plugins>();
    let lock = state.lock().await;
    let keys = lock.keys();
    let keys: Vec<_> = keys.map(|s| s.to_string()).collect();
    Ok(keys)
}

#[tauri::command]
pub async fn load_plugin_ui(
    app: AppHandle,
    plugin: String,
) -> Result<HashMap<String, Vec<u8>>, String> {
    let state = app.state::<Plugins>();
    let mut lock = state.lock().await;
    let assets = lock
        .get_mut(&plugin)
        .ok_or(format!("Plugin {plugin} is not installed"))?
        .get_assets()
        .await
        .map_err(|err| err.to_string())?;
    Ok(assets)
}
