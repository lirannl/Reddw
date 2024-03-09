use chrono::Local;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt::Display, path::PathBuf};
use tauri::{async_runtime::spawn, AppHandle, Manager};
use tokio::io::AsyncWriteExt;
use ts_rs::TS;

use crate::app_handle_ext::AppHandleExt;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize, TS)]
#[ts(export)]
/// Log levels - in ascending order
pub enum LogLevel {
    Debug,
    Info,
    Error,
}

pub fn log(app: &AppHandle, message: &dyn Display, level: LogLevel) {
    let message = message.to_string();
    let app = app.app_handle();
    spawn(async move {
        let behaviours = app.get_config().await.logging;
        for behaviour in behaviours {
            match behaviour {
                LogBehaviour::UIToast(min_level) => {
                    if level < min_level {
                        continue;
                    }
                    app.emit_all("log_message", (message.to_string(), &level))
                        .unwrap_or_default()
                }
                LogBehaviour::StdErr(min_level) => {
                    if level < min_level {
                        continue;
                    }
                    eprintln!("{level:?}: {message}")
                }
                LogBehaviour::StdOut(min_level) => {
                    if level < min_level {
                        continue;
                    }
                    println!("{level:?}: {message}")
                }
                LogBehaviour::File(path, min_level) => {
                    if level < min_level {
                        continue;
                    }
                    let level = level.clone();
                    let message = message.to_string();
                    if let Ok(mut file) = tokio::fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(path)
                        .await
                    {
                        let _ = file
                            .write_all(
                                format!("[{level:?}] {}: {message}\n", Local::now().format("%Y/%m/%d %H:%M")).as_bytes(),
                            )
                            .await;
                    };
                }
            }
        }
    });
}

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub enum LogBehaviour {
    UIToast(LogLevel),
    File(PathBuf, LogLevel),
    StdOut(LogLevel),
    StdErr(LogLevel),
}

pub type LogBehaviours = HashSet<LogBehaviour>;
