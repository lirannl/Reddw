use serde::{Serialize, Deserialize};
use sqlx::{FromRow, types::chrono::NaiveDateTime};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, TS, FromRow)]
#[ts(export)]
pub struct Wallpaper {
    pub id: String,
    pub name: String,
    pub data_url: String,
    pub info_url: Option<String>,
    #[ts(type = "string")]
    pub date: NaiveDateTime,
    pub source: String,
    pub was_set: bool,
}