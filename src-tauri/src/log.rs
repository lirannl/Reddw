use std::fmt::Display;

use serde::Serialize;
use tauri::{AppHandle, Manager};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub enum LogLevel {
    Error,
}

pub fn log(app: &AppHandle, message: &dyn Display, level: LogLevel) {
    app.emit_all("log_message", (message.to_string(), level)).unwrap_or_default();
}
