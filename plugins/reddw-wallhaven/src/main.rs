#![feature(async_closure, let_chains)]
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use reddw_source_plugin::{ReddwSource, Wallpaper};
use reqwest::{Client, Method, Url};
use response_data::BaseResponse;
use rmp_serde::to_vec;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error};
use tokio::sync::Mutex;
mod response_data;

#[derive(RustEmbed)]
#[folder = "ui/dist/"]
struct StaticAssets;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct Parameters {
    search_terms: Vec<String>,
}
impl TryFrom<Vec<u8>> for Parameters {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(rmp_serde::from_slice(&value)?)
    }
}
impl TryInto<Vec<u8>> for Parameters {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        Ok(to_vec(&self)?)
    }
}

struct WallHavenSource {}
impl ReddwSource<Parameters> for WallHavenSource {
    fn get_name() -> Result<String, Box<dyn Error>> {
        Ok(NAME.to_owned())
    }

    fn get_assets() -> Result<HashMap<String, Vec<u8>>, Box<dyn Error>> {
        Ok(StaticAssets::iter()
            .filter_map(|path| {
                let data = StaticAssets::get(&path)?.data.into_owned();
                Some((path.into(), data))
            })
            .collect())
    }

    async fn inspect_instance(id: String) -> Result<Vec<u8>, Box<dyn Error>> {
        let parameters = (*INSTANCES)
            .lock()
            .await
            .get(&id)
            .ok_or(anyhow!("No registered instance under the name \"{id}\""))?
            .clone()
            .try_into()?;
        Ok(parameters)
    }

    async fn register_instance(id: String, parameters: Parameters) -> Result<(), Box<dyn Error>> {
        if id.contains("_") {
            Err(anyhow!(
                "Invalid instance ID. Instance IDs cannot contain underscores"
            ))?
        }
        let mut instances = (*INSTANCES).lock().await;
        instances.insert(id, parameters);
        Ok(())
    }

    async fn deregister_instance(id: String) -> Result<(), Box<dyn Error>> {
        let mut instances = (*INSTANCES).lock().await;
        instances
            .remove(&id)
            .ok_or(anyhow!("No registered instance under the name \"{id}\""))?;
        Ok(())
    }

    async fn get_wallpapers(id: String) -> Result<Vec<Wallpaper>, Box<dyn Error>> {
        let _parameters = {
            let instances = (*INSTANCES).lock().await;
            instances
                .get(&id)
                .ok_or(anyhow!("No registered instance under the name \"{id}\""))?
                .clone()
        };
        let request = reqwest::Request::new(
            Method::GET,
            Url::parse("https://wallhaven.cc/api/v1/search")?,
        );
        let response = Client::new().execute(request).await?;
        if !response.status().is_success() {
            Err(anyhow!(
                "HTTP {} while attempting to communicate with WallHaven",
                response.status()
            ))?;
        }
        let response: BaseResponse = serde_json::from_slice(&response.bytes().await?)?;
        Ok(response
            .data
            .into_iter()
            .map(|datum| {
                Wallpaper::new(
                    datum.id,
                    None,
                    datum.path,
                    Some(datum.url),
                    format!("{NAME}_{id}"),
                )
            })
            .take(2)
            .collect())
    }

    async fn get_instances() -> Result<Vec<String>, Box<dyn Error>> {
        let instances = (*INSTANCES).lock().await;
        Ok(instances
            .keys()
            .clone()
            .into_iter()
            .map(String::to_owned)
            .collect())
    }
}
static NAME: &str = "WallHaven";

lazy_static! {
    static ref INSTANCES: Mutex<HashMap<String, Parameters>> = Mutex::new(HashMap::new());
}

#[tokio::main]
async fn main() -> ! {
    WallHavenSource::serve_plugin().await
}
