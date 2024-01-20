#![feature(async_closure, if_let_guard, never_type, let_chains)]

#[cfg(not(feature = "host"))]
use chrono::NaiveDateTime;
use io_plugin::{Deserialise, Serialise, io_plugin, GenericValue as GenericValueInner};
use serde::{Deserialize, Serialize};
#[cfg(feature = "host")]
use sqlx::{query, types::chrono::NaiveDateTime, FromRow, SqlitePool};
use std::error::Error;
use std::{collections::HashMap, fmt::Debug};
use ts_rs::TS;

#[io_plugin(handle = "host", plugin_trait = "plugin")]
pub enum ReddwSource<Parameters: Serialise + Deserialise> {
    #[implementation(get_version)]
    InterfaceVersion(String),
    /// Gets the source plugin's name
    GetName(String),
    /// Gets the embedded static assets for the application
    GetAssets(HashMap<String, Vec<u8>>),
    /// Check the parameters for a given instance
    InspectInstance(String, Parameters),
    /// Create/modify an instance. Returns whether an existing instance was overridden
    RegisterInstance(String, Parameters, bool),
    /// Remove an instance
    DeregisterInstance(String, ()),
    /// Use an instance to get wallpapers
    GetWallpapers(String, Vec<String>, Vec<Wallpaper>),
    GetInstances(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[cfg_attr(feature = "host", derive(FromRow))]
#[ts(export_to = "../../src-tauri/bindings/")]
#[ts(export)]
pub struct Wallpaper {
    pub id: String,
    pub name: Option<String>,
    pub data_url: String,
    pub info_url: Option<String>,
    #[ts(type = "string")]
    pub date: NaiveDateTime,
    pub source: String,
    pub was_set: bool,
}

impl Wallpaper {
    pub fn new(
        id: String,
        name: Option<String>,
        data_url: String,
        info_url: Option<String>,
        source: String,
    ) -> Self {
        Wallpaper {
            id,
            name,
            data_url,
            info_url,
            date: chrono::Utc::now().naive_utc(),
            was_set: false,
            source,
        }
    }
    #[cfg(feature = "host")]
    pub async fn db_insert(self, db: &SqlitePool) -> Result<(), Box<dyn Error>> {
        query!(
            "---sql
            insert into queue (id, name, data_url, info_url, date, source, was_set) values 
            ($1, $2, $3, $4, $5, $6, $7)",
            self.id,
            self.name,
            self.data_url,
            self.info_url,
            self.date,
            self.source,
            self.was_set,
        )
        .execute(db)
        .await?;
        Ok(())
    }
}

#[cfg(feature = "plugin")]
const INTERFACE_VERSION: &str = env!("CARGO_PKG_VERSION");
#[cfg(feature = "plugin")]
async fn get_version<Parameters: Serialise + Deserialise, Plugin: ReddwSourceTrait<Parameters>>(
    _plugin: &mut Plugin,
) -> Result<String, Box<dyn Error>> {
    Ok(INTERFACE_VERSION.to_string())
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GenericValue(pub GenericValueInner);

impl TS for GenericValue {
    fn name() -> String {
        "any".to_string()
    }

    fn dependencies() -> Vec<ts_rs::Dependency> {
        Vec::with_capacity(0)
    }

    fn transparent() -> bool {
        false
    }
}