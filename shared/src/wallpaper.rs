use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
#[cfg(feature = "sqlx")]
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(FromRow))]
pub struct Wallpaper {
    pub id: String,
    pub name: String,
    pub data_url: String,
    pub info_url: Option<String>,
    pub date: NaiveDateTime,
    #[cfg(not(feature="sqlx"))]
    pub source: Source,
    #[cfg(feature="sqlx")]
    pub source: String,
    #[serde(skip)]
    pub was_set: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Source {
    Subreddit(String),
}
impl Default for Source {
    fn default() -> Self {
        Self::Subreddit("wallpapers".to_string())
    }
}
