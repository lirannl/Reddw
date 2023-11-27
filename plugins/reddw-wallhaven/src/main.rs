#![feature(async_closure, let_chains)]
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use reddw_source_plugin::{ReddwSource, SourceParameterType, SourceParameters, Wallpaper};
use reqwest::{Client, Method, Url};
use response_data::BaseResponse;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error};
use tokio::sync::Mutex;
mod response_data;

#[derive(Serialize, Deserialize)]
struct WallHavenItem {}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct Parameters {}
impl TryFrom<SourceParameters> for Parameters {
    type Error = anyhow::Error;

    fn try_from(_value: SourceParameters) -> Result<Self, Self::Error> {
        Ok(Parameters {})
    }
}
impl TryInto<SourceParameters> for Parameters {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<SourceParameters, Self::Error> {
        Ok(HashMap::new())
    }
}

struct WallHavenSource {}
impl ReddwSource<Parameters> for WallHavenSource {
    fn get_name() -> Result<String, Box<dyn Error>> {
        Ok(NAME.to_owned())
    }

    fn get_parameters() -> Result<HashMap<String, SourceParameterType>, Box<dyn Error>> {
        Ok(HashMap::<String, SourceParameterType>::new())
    }

    async fn inspect_instance(id: String) -> Result<SourceParameters, Box<dyn Error>> {
        let parameters = (*INSTANCES)
            .lock()
            .await
            .get(&id)
            .ok_or(anyhow!("No registered instance under the name \"{id}\""))?
            .clone()
            .try_into()?;
        Ok(parameters)
    }

    async fn register_instance(id: String, params: SourceParameters) -> Result<(), Box<dyn Error>> {
        if id.contains("_") {
            Err(anyhow!(
                "Invalid instance ID. Instance IDs cannot contain underscores"
            ))?
        }
        let parameters = params.try_into()?;
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
    loop {
        WallHavenSource::main_loop().await.unwrap_or_else(|_| {})
    }
}
