use std::{time::Duration, path::PathBuf};

use serde::{Serialize, Deserialize};

use crate::Source;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    /// Allow fetching wallpapers from Not Safe For Work sources (aka - sexually explicit content/gore)
    pub allow_nsfw: bool,
    pub sources: Vec<Source>,
    /// How often to switch new wallpapers (in seconds)
    pub interval: Duration,
    pub cache_dir: PathBuf,
    // Max cache size, in megabytes
    pub cache_size: f64,
    pub history_amount: i32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            allow_nsfw: false,
            sources: vec![Default::default()],
            interval: Duration::from_secs(60 * 60),
            cache_dir: PathBuf::new(),
            cache_size: 100.0,
            history_amount: 10,
        }
    }
}