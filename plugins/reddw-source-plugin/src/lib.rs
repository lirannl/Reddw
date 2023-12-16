#![feature(async_closure, if_let_guard, never_type, let_chains)]

use std::{
    collections::HashMap,
    error::Error,
    fmt::Debug,
    future::Future,
    io::{stdin, stdout, ErrorKind, Write},
    process::exit,
};

#[cfg(not(sqlx))]
use chrono::NaiveDateTime;
use rmp_serde::{encode::write, from_read, from_slice, to_vec};
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
    InterfaceVersion,
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
    Err(String),
    InterfaceVersion(String),
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

pub const INTERFACE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub trait ReddwSource<Parameters: Debug + for<'a> Deserialize<'a> + Send> {
    /// Gets the plugin's interface version
    fn interface_version() -> String {
        INTERFACE_VERSION.to_string()
    }
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
        params: Parameters,
    ) -> impl Future<Output = Result<(), Box<dyn Error>>> + Send;
    /// Remove an instance
    fn deregister_instance(id: String) -> impl Future<Output = Result<(), Box<dyn Error>>> + Send;
    /// Use an instance to get wallpapers
    fn get_wallpapers(
        id: String,
    ) -> impl Future<Output = Result<Vec<Wallpaper>, Box<dyn Error>>> + Send;
    fn get_instances() -> impl Future<Output = Result<Vec<String>, Box<dyn Error>>> + Send;
    fn serve_plugin() -> impl Future<Output = !> + Send
    where
        Self: Sized,
    {
        async {
            loop {
                let response = loop_iter::<Parameters, Self>()
                    .await
                    .unwrap_or_else(|err| SourcePluginResponse::Err(err.to_string()));
                write(&mut stdout(), &response).unwrap_or_else(|err| {
                    eprintln!("Couldn't send response: {err}");
                });
                let status = stdout().flush();
                if let Err(err) = status
                    && err.kind() == ErrorKind::BrokenPipe
                {
                    exit(1);
                }
            }
        }
    }
}

async fn loop_iter<
    Parameters: Debug + for<'a> Deserialize<'a> + Send,
    Source: ReddwSource<Parameters>,
>() -> Result<SourcePluginResponse, Box<dyn Error>> {
    let received = from_read::<_, SourcePluginMessage>(stdin())?;
    match received {
        SourcePluginMessage::InterfaceVersion => Ok(SourcePluginResponse::InterfaceVersion(
            Source::interface_version(),
        )),
        SourcePluginMessage::GetName => Ok(SourcePluginResponse::GetName(Source::get_name()?)),
        SourcePluginMessage::GetAssets => {
            Ok(SourcePluginResponse::GetAssets(Source::get_assets()?))
        }
        SourcePluginMessage::InspectInstance(id) => Ok(SourcePluginResponse::InspectInstance(
            Source::inspect_instance(id).await?,
        )),
        SourcePluginMessage::RegisterInstance(id, params) => {
            Source::register_instance(id, from_slice(&params)?).await?;
            Ok(SourcePluginResponse::RegisterInstance)
        }
        SourcePluginMessage::DeregisterInstance(id) => {
            Source::deregister_instance(id).await?;
            Ok(SourcePluginResponse::DeregisterInstance)
        }
        SourcePluginMessage::GetWallpapers(id) => Ok(SourcePluginResponse::GetWallpapers(
            Source::get_wallpapers(id).await?,
        )),
        SourcePluginMessage::GetInstances => Ok(SourcePluginResponse::GetInstances(
            Source::get_instances().await?,
        )),
    }
}
