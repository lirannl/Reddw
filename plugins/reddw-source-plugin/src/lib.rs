#![feature(async_closure, if_let_guard, never_type, let_chains)]

use std::{collections::HashMap, fmt::Debug};

#[cfg(not(sqlx))]
use chrono::NaiveDateTime;
use io_plugin::io_plugin;
use io_plugin::{Deserialise, Serialise};
use serde::{Deserialize, Serialize};
#[cfg(plugin)]
use std::error::Error;
#[cfg(sqlx)]
use sqlx::{types::chrono::NaiveDateTime, FromRow};
use ts_rs::TS;

#[io_plugin(handle = "host", plugin_trait = "plugin")]
pub enum ReddwSource<Parameters: Serialise + Deserialise> {
    InterfaceVersion(String),
    /// Gets the source plugin's name
    GetName(String),
    /// Gets the embedded static assets for the application
    GetAssets(HashMap<String, Vec<u8>>),
    /// Check the parameters for a given instance
    InspectInstance(String, Parameters),
    /// Create/modify an instance
    RegisterInstance(String, Parameters, ()),
    /// Remove an instance
    DeregisterInstance(String, ()),
    /// Use an instance to get wallpapers
    GetWallpapers(String, Vec<Wallpaper>),
    GetInstances(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[cfg_attr(sqlx, derive(FromRow))]
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
            date: NaiveDateTime::default(),
            was_set: false,
            source,
        }
    }
}

pub const INTERFACE_VERSION: &str = env!("CARGO_PKG_VERSION");