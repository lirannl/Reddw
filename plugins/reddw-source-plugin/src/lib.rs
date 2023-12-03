#![feature(async_closure)]

use std::{
    collections::HashMap,
    error::Error,
    fmt::Debug,
    future::Future,
    io::{stderr, stdin, stdout, Write},
};

#[cfg(not(sqlx))]
use chrono::NaiveDateTime;
use rmp_serde::{encode::write, from_read};
use serde::{Deserialize, Serialize};
#[cfg(sqlx)]
use sqlx::{types::chrono::NaiveDateTime, FromRow};
use ts_rs::TS;

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

#[derive(Debug, Serialize, Deserialize)]
pub enum SourcePluginMessage {
    /// Gets the source plugin's name
    GetName,
    /// Gets the embedded static assets for the application
    GetAssets,
    /// Check the parameters for a given instance
    InspectInstance(String),
    /// Create/modify an instance
    RegisterInstance(String, Vec<u8>),
    /// Remove an instance
    DeregisterInstance(String),
    /// Use an instance to get wallpapers
    GetWallpapers(String),
    GetInstances,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SourcePluginResponse {
    /// Gets the source plugin's name
    GetName(String),
    /// Gets the embedded static assets for the application
    GetAssets(HashMap<String, Vec<u8>>),
    /// Check the parameters for a given instance
    InspectInstance(Vec<u8>),
    /// Create/modify an instance
    RegisterInstance,
    /// Remove an instance
    DeregisterInstance,
    /// Use an instance to get wallpapers
    GetWallpapers(Vec<Wallpaper>),
    GetInstances(Vec<String>),
}

pub trait ReddwSource<Parameters> {
    /// Gets the source plugin's name
    fn get_name() -> Result<String, Box<dyn Error>>;
    /// Gets the embedded static assets for the application
    fn get_assets() -> Result<HashMap<String, Vec<u8>>, Box<dyn Error>>;
    /// Check the parameters for a given instance
    fn inspect_instance(id: String)
        -> impl Future<Output = Result<Vec<u8>, Box<dyn Error>>> + Send;
    /// Create/modify an instance
    fn register_instance(
        id: String,
        params: Vec<u8>,
    ) -> impl Future<Output = Result<(), Box<dyn Error>>> + Send;
    /// Remove an instance
    fn deregister_instance(id: String) -> impl Future<Output = Result<(), Box<dyn Error>>> + Send;
    /// Use an instance to get wallpapers
    fn get_wallpapers(
        id: String,
    ) -> impl Future<Output = Result<Vec<Wallpaper>, Box<dyn Error>>> + Send;
    fn get_instances() -> impl Future<Output = Result<Vec<String>, Box<dyn Error>>> + Send;
    fn main_loop() -> impl Future<Output = Result<(), Box<dyn Error>>> + Send {
        async {
            let result: Result<SourcePluginResponse, Box<dyn Error>> =
                match from_read::<_, SourcePluginMessage>(stdin())? {
                    SourcePluginMessage::GetName => {
                        Ok(SourcePluginResponse::GetName(Self::get_name()?))
                    }
                    SourcePluginMessage::GetAssets => {
                        Ok(SourcePluginResponse::GetAssets(Self::get_assets()?))
                    }
                    SourcePluginMessage::InspectInstance(id) => Ok(
                        SourcePluginResponse::InspectInstance(Self::inspect_instance(id).await?),
                    ),
                    SourcePluginMessage::RegisterInstance(id, params) => {
                        Self::register_instance(id, params.try_into()?).await?;
                        Ok(SourcePluginResponse::RegisterInstance)
                    }
                    SourcePluginMessage::DeregisterInstance(id) => {
                        Self::deregister_instance(id).await?;
                        Ok(SourcePluginResponse::DeregisterInstance)
                    }
                    SourcePluginMessage::GetWallpapers(id) => Ok(
                        SourcePluginResponse::GetWallpapers(Self::get_wallpapers(id).await?),
                    ),
                    SourcePluginMessage::GetInstances => Ok(SourcePluginResponse::GetInstances(
                        Self::get_instances().await?,
                    )),
                };
            match result {
                Ok(response) => {
                    write(&mut stdout(), &response).unwrap_or_else(|err| eprintln!("{err}"))
                }
                Err(error) => eprint!("{error}"),
            }
            eprint!("\n");
            stdout().flush()?;
            stderr().flush()?;
            Ok(())
        }
    }
}
