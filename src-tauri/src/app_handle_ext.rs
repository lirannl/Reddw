use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use crate::{
    app_config::AppConfig,
    log::{log as log_func, LogLevel},
    queue::DB,
};
use anyhow::Result;
use reddw_ipc::{IPCData, IPCMessage};
use rmp_serde::from_slice;
use serde::Deserialize;
use tauri::{async_runtime::Mutex, AppHandle, Manager};
use tokio::sync::watch::Receiver;

pub trait AppHandleExt {
    fn get_config_path(&self) -> PathBuf;
    async fn get_config(&self) -> AppConfig;
    async fn db(&self) -> DB;
    fn log(&self, message: &dyn Display, level: LogLevel) -> ();
    async fn listen_ipc<T: for<'a> Deserialize<'a>>(
        &self,
        message_filter: impl Fn(&IPCMessage) -> bool,
    ) -> Result<T>;
}

impl AppHandleExt for AppHandle {
    fn get_config_path(&self) -> PathBuf {
        let config_dir = self.path_resolver().app_config_dir().unwrap();
        Path::join(&config_dir, "config.json")
    }
    async fn get_config(&self) -> AppConfig {
        self.state::<Mutex<AppConfig>>().lock().await.clone()
    }
    async fn db(&self) -> DB {
        self.state::<Mutex<DB>>().lock().await.clone()
    }

    fn log(&self, message: &dyn Display, level: LogLevel) -> () {
        log_func(self, message, level)
    }

    async fn listen_ipc<T: for<'a> Deserialize<'a>>(
        &self,
        message_filter: impl Fn(&IPCMessage) -> bool,
    ) -> Result<T> {
        let mut receiver = self.state::<Receiver<IPCData<Vec<u8>>>>().inner().clone();
        let message = receiver
            .wait_for(|message| {
                (|| match message {
                    (IPCMessage::Init, _) => None,
                    (t, _) if message_filter(t) => Some(()),
                    _ => None,
                })()
                .is_some()
            })
            .await
            .unwrap()
            .to_owned();
        let _ = receiver.changed().await;
        Ok(from_slice(&message.1)?)
    }
}
