use std::path::{Path, PathBuf};

use tauri::{async_runtime::Mutex, AppHandle, Manager};

use crate::{app_config::AppConfig, queue::DB};

pub trait AppHandleExt {
    fn get_config_path(&self) -> PathBuf;
    async fn get_config(&self) -> AppConfig;
    async fn db(&self) -> DB;
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
}
