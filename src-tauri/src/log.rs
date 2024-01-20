use std::fmt::Display;

use serde::Serialize;
use tauri::{AppHandle, Manager};
use ts_rs::TS;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, TS)]
#[ts(export)]
pub enum LogLevel {
    Info,
    Error,
    Debug,
}

pub fn log(app: &AppHandle, message: &dyn Display, level: LogLevel) {
    app.emit_all("log_message", (message.to_string(), level))
        .unwrap_or_default();
}
