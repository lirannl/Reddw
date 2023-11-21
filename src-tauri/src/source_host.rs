use anyhow::{anyhow, bail, Result};
use reddw_source_plugin::{
    SourceParameter, SourceParameterType, SourceParameters, SourcePluginMessage,
    SourcePluginResponse, Wallpaper,
};
use serde::{Deserialize, Serialize};
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::{collections::HashMap, fs::read_dir, mem::size_of, process::Stdio};
use tauri::{async_runtime::block_on, AppHandle, Manager};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    join,
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command},
    sync::Mutex,
};
use ts_rs::TS;

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
pub enum PluginHostMode {
    Daemon,
    LowRAM,
}

pub struct PluginHandle {
    process: Child,
    stdin: Mutex<ChildStdin>,
    stdout: Mutex<ChildStdout>,
    stderr: Mutex<ChildStderr>,
    pub name: String,
}
impl PluginHandle {
    async fn new(mut child_process: Child) -> Result<PluginHandle> {
        let mut new_handle = ((|| {
            Some(PluginHandle {
                stdin: Mutex::new(child_process.stdin.take()?),
                stdout: Mutex::new(child_process.stdout.take()?),
                stderr: Mutex::new(child_process.stderr.take()?),
                process: child_process,
                name: "".to_string(),
            })
        })()
        .ok_or(anyhow!(
            "Invalid plugin - stdin, stdout, and stderr could not be established."
        )))?;
        new_handle.name = new_handle.get_name().await?;
        Ok(new_handle)
    }
    async fn message(&mut self, message: SourcePluginMessage) -> Result<SourcePluginResponse> {
        let (mut stdin, mut stdout, mut stderr) =
            join!(self.stdin.lock(), self.stdout.lock(), self.stderr.lock());
        stdin.write(&rmp_serde::to_vec(&message)?).await?;
        let err = {
            let mut err = Vec::<u8>::new();
            stderr.read(&mut err).await.unwrap_or_default();
            String::from_utf8(err).unwrap_or_default()
        };
        let result: SourcePluginResponse = {
            let mut buf = [0 as u8; size_of::<SourcePluginResponse>()];
            stdout.read(&mut buf).await?;
            rmp_serde::from_slice(&buf)
        }?;
        if err != "" {
            bail!("{err}")
        }
        Ok(result)
    }
    async fn get_name(&mut self) -> Result<String> {
        match self.message(SourcePluginMessage::GetName).await? {
            SourcePluginResponse::GetName(name) => Ok(name),
            _ => Err(anyhow!("Invalid response")),
        }
    }
    #[allow(dead_code)]
    pub async fn get_parameters(&mut self) -> Result<HashMap<String, SourceParameterType>> {
        match self.message(SourcePluginMessage::GetParameters).await? {
            SourcePluginResponse::GetParameters(parameters) => Ok(parameters),
            _ => Err(anyhow!("Invalid response")),
        }
    }
    pub async fn register_instance(
        &mut self,
        id: String,
        parameters: HashMap<String, SourceParameter>,
    ) -> Result<()> {
        match self
            .message(SourcePluginMessage::RegisterInstance(id, parameters))
            .await?
        {
            SourcePluginResponse::RegisterInstance => Ok(()),
            _ => Err(anyhow!("Invalid response")),
        }
    }
    #[allow(dead_code)]
    pub async fn inspect_instance(&mut self, id: String) -> Result<SourceParameters> {
        match self
            .message(SourcePluginMessage::InspectInstance(id))
            .await?
        {
            SourcePluginResponse::InspectInstance(parameters) => Ok(parameters),
            _ => Err(anyhow!("Invalid response")),
        }
    }
    pub async fn get_wallpapers(&mut self, id: String) -> Result<Vec<Wallpaper>> {
        match self.message(SourcePluginMessage::GetWallpapers(id)).await? {
            SourcePluginResponse::GetWallpapers(wallpapers) => Ok(wallpapers),
            _ => Err(anyhow!("Invalid response")),
        }
    }
}
impl Drop for PluginHandle {
    fn drop(&mut self) {
        let _ = block_on(self.process.kill());
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
                println!("Loading plugins from: {:#?}", path);
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
    for plugin in read_dir(plugins_dir)?.filter_map(|f| f.ok()) {
        // Check execute bit
        #[cfg(target_family = "unix")]
        if plugin.metadata()?.permissions().mode() % 2 != 0 {
            continue;
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
        let name = plugin.message(SourcePluginMessage::GetName).await?;
        let name = match name {
            SourcePluginResponse::GetName(name) => Ok(name),
            _ => Err(anyhow!(
                "Couldn't get source plugin name for the plugin \"{:?}\"",
                plugin_file
            )),
        }?;
        let instances = config
            .sources
            .clone()
            .into_iter()
            .filter_map(|(key, parameters)| {
                if let Some((plugin, instance)) = key.split_once("_")
                    && plugin == name
                {
                    Some((instance.to_string(), parameters))
                } else {
                    None
                }
            });
        for (id, params) in instances {
            plugin.register_instance(id.to_string(), params).await?;
        }
        plugins.insert(name, plugin);
    }
    app.manage::<Plugins>(Mutex::new(plugins));
    Ok(())
}
