use anyhow::{anyhow, bail, Result};
use reddw_source_plugin::{
    SourceParameter, SourceParameterType, SourcePluginMessage, SourcePluginResponse,
};
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::{collections::HashMap, fs::read_dir, io::Stdout, process::Stdio};
use tauri::{AppHandle, Manager};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command},
    sync::Mutex,
};

struct PluginHandle {
    stdin: ChildStdin,
    stdout: ChildStdout,
    stderr: ChildStderr,
    pub name: String,
    kill: Box<dyn FnOnce() + Send>,
}
impl PluginHandle {
    async fn new(mut child_process: Child) -> Result<PluginHandle> {
        let mut new_handle = ((|| {
            Some(PluginHandle {
                stdin: child_process.stdin.take()?,
                stdout: child_process.stdout.take()?,
                stderr: child_process.stderr.take()?,
                name: "".to_string(),
                kill: Box::new(move || {
                    child_process.kill();
                }),
            })
        })()
        .ok_or(anyhow!(
            "Invalid plugin - stdin, stdout, and stderr could not be established."
        )))?;
        new_handle.name = new_handle.get_name().await?;
        Ok(new_handle)
    }
    async fn message(&mut self, message: SourcePluginMessage) -> Result<SourcePluginResponse> {
        self.stdin.write(&rmp_serde::to_vec(&message)?).await?;
        let err = {
            let mut err = String::new();
            self.stderr
                .read_to_string(&mut err)
                .await
                .unwrap_or_default();
            err
        };
        let result: SourcePluginResponse = {
            let mut buf = Vec::new();
            self.stdout.read(&mut buf);
            rmp_serde::from_slice(&buf)
        }?;
        if err != "" {
            bail!(err)
        }
        Ok(result)
    }
    async fn get_name(&mut self) -> Result<String> {
        match self.message(SourcePluginMessage::GetName).await? {
            SourcePluginResponse::GetName(name) => Ok(name),
            _ => Err(anyhow!("Invalid response")),
        }
    }
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
}
impl Drop for PluginHandle {
    fn drop(&mut self) {
        self.kill.as_mut()()
    }
}

use crate::app_handle_ext::AppHandleExt;

pub async fn host_plugins(app: AppHandle) -> Result<()> {
    let config = app.get_config().await;
    let plugins_dir = {
        match config.plugins_dir {
            Some(path) => path,
            None => app
                .path_resolver()
                .app_config_dir()
                .expect("App config folder could't be determined")
                .join("plugins"),
        }
    };
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
        plugins.insert(name, plugin);
    }
    app.manage(Mutex::new(plugins));
    Ok(())
}
