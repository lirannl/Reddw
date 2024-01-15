use anyhow::{anyhow, bail, Result};
use macros::command;
use reddw_source_plugin::ReddwSourceHandle;
use serde::{Deserialize, Serialize};
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::{
    collections::HashMap,
    fs::read_dir,
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Manager};
use tokio::{fs, sync::Mutex};
use ts_rs::TS;

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub enum PluginHostMode {
    Daemon,
    LowRAM,
}

use crate::{app_handle_ext::AppHandleExt, log::LogLevel};

type PluginMap = HashMap<String, ReddwSourceHandle>;

pub type SourcePlugins = Mutex<PluginMap>;

pub async fn host_sources(app: AppHandle) -> Result<()> {
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

    let mut plugins = PluginMap::new();

    load_plugins(app.app_handle(), &mut plugins, &plugins_dir).await?;

    app.manage::<SourcePlugins>(Mutex::new(plugins));

    // let mut watcher = recommended_watcher(move |event: notify::Result<notify::Event>| {
    //     let app = app.app_handle();
    //     (|| -> Result<()> {
    //         let event = event?;
    //         let plugins = app.state::<SourcePlugins>();
    //         let mut plugins = plugins.blocking_lock();
    //         if event.kind.is_remove() || event.kind.is_modify() {
    //             let names = plugins
    //                 .iter_mut()
    //                 .filter_map(|(_, plugin)| {
    //                     if event.paths.contains(&plugin.path) {
    //                         Some(plugin.name.clone())
    //                     } else {
    //                         None
    //                     }
    //                 })
    //                 .collect::<Vec<_>>();
    //             for name in names {
    //                 plugins.remove(&name);
    //             }
    //         }
    //         if event.kind.is_create() || event.kind.is_modify() {
    //             for path in event.paths {
    //                 block_on(load_plugin(app.app_handle(), path, &mut plugins))
    //                     .map(|name| app.log(&format!("Loaded \"{name}\""), LogLevel::Info))
    //                     .unwrap_or_else(|err| app.log(&err, LogLevel::Error));
    //             }
    //         }
    //         Ok(())
    //     })()
    //     .unwrap_or_else(move |err| app.log(&err, LogLevel::Error));
    // })?;
    // watcher.watch(plugins_dir.as_path(), RecursiveMode::NonRecursive)?;

    Ok(())
}

pub async fn load_plugins(
    app: AppHandle,
    plugins: &mut PluginMap,
    plugins_dir: &Path,
) -> Result<()> {
    if let Ok(exists) = fs::try_exists(&plugins_dir).await
        && !exists
    {
        fs::create_dir_all(&plugins_dir).await?;
    }
    let plugin_files = read_dir(plugins_dir)?.filter_map(|f| f.ok());
    for plugin in plugin_files {
        let plugin_name = format!("{:?}", plugin.file_name());
        let plugin_name_clone = plugin_name.clone();
        let plugins = &mut *plugins;
        let app_handle = app.app_handle();
        (async move || -> Result<()> {
            let app = app_handle;
            // Check execute bit
            #[cfg(target_family = "unix")]
            {
                let permissions: Vec<u8> = format!("{:o}", plugin.metadata()?.permissions().mode())
                    [3..]
                    .chars()
                    .filter_map(|c| format!("{c}").parse().ok())
                    .collect();
                if permissions.iter().all(|p| p % 2 == 0) {
                    bail!("{plugin_name} lacks execute permissions")
                }
            }
            load_plugin(app.app_handle(), plugin.path(), plugins).await?;
            Ok(())
        })()
        .await
        .unwrap_or_else(|err| {
            app.log(
                &format!("Couldn't load plugin {plugin_name_clone}: {err:#?}"),
                LogLevel::Error,
            );
        });
    }
    Ok(())
}

async fn load_plugin(
    app: AppHandle,
    plugin: PathBuf,
    plugins: &mut PluginMap,
) -> Result<String, anyhow::Error> {
    let sources = app.get_config().await.sources;
    let mut name = plugin
        .clone()
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or(anyhow!("Invalid path {plugin:#?}"))?
        .to_string();
    let mut plugin = ReddwSourceHandle::new(plugin)
        .await
        .map_err(|err| anyhow!("Couldn't spawn plugin {name} {err:#?}"))?;
    name = plugin.name.clone();
    let instances = sources.into_iter().filter_map(|(key, parameters)| {
        if let Some((plugin, instance)) = key.split_once("_")
            && plugin == name
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

    let name = plugin.name.to_string();
    plugins.insert(plugin.name.to_string(), plugin);

    Ok(name)
}

#[command]
pub async fn query_available_source_plugins(app: AppHandle) -> Result<Vec<String>> {
    let state = app.state::<SourcePlugins>();
    let lock = state.lock().await;
    let keys = lock.keys();
    let keys: Vec<_> = keys.map(|s| s.to_string()).collect();
    Ok(keys)
}

#[command]
pub async fn load_plugin_ui(app: AppHandle, plugin: String) -> Result<HashMap<String, Vec<u8>>> {
    let state = app.state::<SourcePlugins>();
    let mut lock = state.lock().await;
    let assets = lock
        .get_mut(&plugin)
        .ok_or(anyhow!("Plugin {plugin} is not installed"))?
        .get_assets()
        .await
        .map_err(|err| anyhow!(err.to_string()))?;
    Ok(assets)
}
