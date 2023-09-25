#![feature(
    async_closure,
    async_fn_in_trait)]

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

impl Wallpaper {
    pub fn new(id: String, name: String, data_url: String, info_url: Option<String>, source: String) -> Self { Wallpaper {
    id, name, data_url, info_url, date: NaiveDateTime::default(), was_set: false, source
    }}
}
pub trait SourcePlugin {
    async fn get_wallpapers(&self) -> Vec<Wallpaper>;
}