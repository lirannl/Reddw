use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BaseResponse {
    pub data: Vec<Datum>,
    pub meta: Meta,
}

#[derive(Serialize, Deserialize)]
pub struct Datum {
    pub id: String,
    pub url: String,
    pub short_url: String,
    pub views: u64,
    pub favorites: u64,
    pub source: String,
    pub purity: Purity,
    pub category: Category,
    pub dimension_x: u64,
    pub dimension_y: u64,
    pub resolution: String,
    pub ratio: String,
    pub file_size: u64,
    pub file_type: FileType,
    pub created_at: String,
    pub colors: Vec<String>,
    pub path: String,
    pub thumbs: Option<Thumbs>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Anime,
    General,
}

#[derive(Serialize, Deserialize)]
pub enum FileType {
    #[serde(rename = "image/jpeg")]
    ImageJpeg,
    #[serde(rename = "image/png")]
    ImagePng,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Purity {
    Sfw,
}

#[derive(Serialize, Deserialize)]
pub struct Thumbs {
    pub large: Option<String>,
    pub original: String,
    pub small: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Meta {
    pub current_page: u64,
    pub last_page: u64,
    pub per_page: u64,
    pub total: u64,
    query: Option<serde_json::Value>,
    seed: Option<serde_json::Value>,
}
