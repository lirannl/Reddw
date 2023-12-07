use anyhow::{anyhow, bail, Result};
#[cfg(target_os = "linux")]
use nix::{sys::signal, unistd::Pid};
use reddw_source_plugin::{SourcePluginMessage, SourcePluginResponse, Wallpaper};
use rmp_serde::from_slice;
use serde::{Deserialize, Serialize};
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::{collections::HashMap, fs::read_dir, process::Stdio, time::Duration};
use tauri::{AppHandle, Manager};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    join,
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command},
    sync::Mutex,
    time::sleep,
};
use ts_rs::TS;

/// Size of plugin-reading buffer
const PLUGIN_BUF_SIZE: usize = 4000;

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub enum PluginHostMode {
    Daemon,
    LowRAM,
}

pub struct PluginHandle {
    // process: Child,
    pid: u32,
    stdin: Mutex<ChildStdin>,
    stdout: Mutex<ChildStdout>,
    stderr: Mutex<ChildStderr>,
    pub name: String,
}
impl PluginHandle {
    async fn new(mut child_process: Child) -> Result<PluginHandle> {
        let mut new_handle = ((|| {
            Some(PluginHandle {
                pid: child_process.id()?,
                stdin: Mutex::new(child_process.stdin.take()?),
                stdout: Mutex::new(child_process.stdout.take()?),
                stderr: Mutex::new(child_process.stderr.take()?),
                // process: child_process,
                name: "".to_string(),
            })
        })()
        .ok_or(anyhow!(
            "Invalid plugin - pid, stdin, stdout, and stderr could not be established."
        )))?;
        new_handle.name = new_handle.get_name().await?;
        Ok(new_handle)
    }
    async fn message(&mut self, message: SourcePluginMessage) -> Result<SourcePluginResponse> {
        let (mut stdin, mut stdout, mut stderr) =
            join!(self.stdin.lock(), self.stdout.lock(), self.stderr.lock());
        Ok((message_internal(&mut stdin, &mut stderr, &mut stdout, &message).await)?)
    }
    async fn get_name(&mut self) -> Result<String> {
        match self.message(SourcePluginMessage::GetName).await? {
            SourcePluginResponse::GetName(name) => Ok(name),
            _ => Err(anyhow!("Invalid response")),
        }
    }
    async fn get_assets(&mut self) -> Result<HashMap<String, Vec<u8>>> {
        match self.message(SourcePluginMessage::GetAssets).await? {
            SourcePluginResponse::GetAssets(assets) => Ok(assets),
            _ => Err(anyhow!("Invalid response")),
        }
    }
    pub async fn register_instance(&mut self, id: String, parameters: Vec<u8>) -> Result<()> {
        match self
            .message(SourcePluginMessage::RegisterInstance(id, parameters))
            .await?
        {
            SourcePluginResponse::RegisterInstance => Ok(()),
            _ => Err(anyhow!("Invalid response")),
        }
    }
    // pub async fn inspect_instance(&mut self, id: String) -> Result<Vec<u8>> {
    //     match self
    //         .message(SourcePluginMessage::InspectInstance(id))
    //         .await?
    //     {
    //         SourcePluginResponse::InspectInstance(parameters) => Ok(parameters),
    //         _ => Err(anyhow!("Invalid response")),
    //     }
    // }
    pub async fn get_wallpapers(&mut self, id: String) -> Result<Vec<Wallpaper>> {
        match self.message(SourcePluginMessage::GetWallpapers(id)).await? {
            SourcePluginResponse::GetWallpapers(wallpapers) => Ok(wallpapers),
            _ => Err(anyhow!("Invalid response")),
        }
    }
}

async fn message_internal(
    stdin: &mut ChildStdin,
    stderr: &mut ChildStderr,
    stdout: &mut ChildStdout,
    message: &SourcePluginMessage,
) -> Result<SourcePluginResponse, anyhow::Error> {
    stdin.write(&rmp_serde::to_vec(&message)?).await?;
    stdin.flush().await?;
    let err = {
        let mut err = Vec::<u8>::new();
        stderr.read(&mut err).await.unwrap_or_default();
        String::from_utf8(err).unwrap_or_default()
    };
    let result: SourcePluginResponse = {
        let mut vec = Vec::<u8>::new();
        let mut buf = [0 as u8; PLUGIN_BUF_SIZE];
        while let Ok(read) = stdout.read(&mut buf).await {
            vec.write(buf.take(PLUGIN_BUF_SIZE as u64).get_ref())
                .await?;
            if read < PLUGIN_BUF_SIZE {
                break;
            }
        }
        from_slice(&vec)
    }?;
    if err != "" {
        bail!("{err}")
    }
    Ok(result)
}
impl Drop for PluginHandle {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        Process::from_id(pid).unwrap().terminate(1);
        #[cfg(target_os = "linux")]
        let _ = signal::kill(Pid::from_raw(self.pid as i32), signal::SIGKILL);
    }
}

use crate::app_handle_ext::AppHandleExt;

pub type Plugins = Mutex<HashMap<String, PluginHandle>>;

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
        // Check execute bit
        #[cfg(target_family = "unix")]
        {
            let permissions: Vec<u8> = format!("{:o}", plugin.metadata()?.permissions().mode())
                [3..]
                .chars()
                .filter_map(|c| format!("{c}").parse().ok())
                .collect();
            if permissions.iter().all(|p| p % 2 == 0) {
                continue;
            }
        }
        let plugin_file = plugin.file_name();
        let mut plugin = PluginHandle::new(
            Command::new(plugin.path())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?,
        )
        .await
        .map_err(|_| {
            anyhow!(
                "Error occurred while initiating plugin \"{:?}\"",
                plugin_file
            )
        })?;
        let name = plugin.get_name().await?;
        // let instances = config
        //     .sources
        //     .clone()
        //     .into_iter()
        //     .filter_map(|(key, parameters)| {
        //         if let Some((plugin, instance)) = key.split_once("_")
        //             && plugin == name
        //         {
        //             Some((instance.to_string(), parameters))
        //         } else {
        //             None
        //         }
        //     });
        // for (id, params) in instances {
        //     plugin
        //         .register_instance(id.to_string(), rmp_serde::to_vec(&params)?)
        //         .await?;
        // }
        plugins.insert(name, plugin);
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
