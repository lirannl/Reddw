#![feature(async_closure, let_chains)]
#![windows_subsystem = "windows"]
use anyhow::{anyhow, Result};
use reddw_source_plugin::{ReddwSourceTrait, Wallpaper};
use reqwest::{Client, Method, Url};
use response_data::BaseResponse;
use rmp_serde::to_vec;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error};
mod response_data;

#[derive(RustEmbed)]
#[folder = "ui/dist/"]
struct StaticAssets;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Parameters {
    tags: Vec<String>,
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

struct WallHavenSource {
    instances: HashMap<String, Parameters>,
}

impl ReddwSourceTrait<Parameters> for WallHavenSource {
    async fn get_name(&mut self) -> Result<String, Box<dyn Error>> {
        Ok(NAME.to_owned())
    }

    async fn get_assets(&mut self) -> Result<HashMap<String, Vec<u8>>, Box<dyn Error>> {
        Ok(StaticAssets::iter()
            .filter_map(|path| {
                let data = StaticAssets::get(&path)?.data.into_owned();
                Some((path.into(), data))
            })
            .collect())
    }

    async fn inspect_instance(&mut self, id: String) -> Result<Parameters, Box<dyn Error>> {
        let parameters = self
            .instances
            .get(&id)
            .ok_or(anyhow!("No registered instance under the name \"{id}\""))?
            .clone()
            .try_into()?;
        Ok(parameters)
    }

    async fn register_instance(
        &mut self,
        id: String,
        parameters: Parameters,
    ) -> Result<(), Box<dyn Error>> {
        if id.contains("_") {
            Err(anyhow!(
                "Invalid instance ID. Instance IDs cannot contain underscores"
            ))?
        }
        self.instances.insert(id, parameters);
        Ok(())
    }

    async fn deregister_instance(&mut self, id: String) -> Result<(), Box<dyn Error>> {
        self.instances
            .remove(&id)
            .ok_or(anyhow!("No registered instance under the name \"{id}\""))?;
        Ok(())
    }

    async fn get_wallpapers(
        &mut self,
        id: String,
        wallpaper_ids: Vec<String>,
    ) -> Result<Vec<Wallpaper>, Box<dyn Error>> {
        let parameters = {
            self.instances
                .get(&id)
                .ok_or(anyhow!("No registered instance under the name \"{id}\""))?
                .clone()
        };
        let source = format!("{NAME}_{id}");
        let mut wallpapers = Vec::new();
        let mut page = 1;
        while wallpapers.len() == 0 {
            wallpapers = wallpapers_page(&source, &parameters, &wallpaper_ids, page).await?;
            page += 1;
        }
        Ok(wallpapers)
    }

    async fn get_instances(&mut self) -> Result<Vec<String>, Box<dyn Error>> {
        Ok(self
            .instances
            .keys()
            .clone()
            .into_iter()
            .map(String::to_owned)
            .collect())
    }
}
static NAME: &str = "WallHaven";

#[tokio::main]
async fn main() {
    WallHavenSource {
        instances: HashMap::new(),
    }
    .main_loop()
    .await
}

async fn wallpapers_page(
    source: &str,
    parameters: &Parameters,
    ids: &Vec<String>,
    page: u32,
) -> Result<Vec<Wallpaper>, Box<dyn Error>> {
    let terms = parameters.tags.join("+");
    let request = reqwest::Request::new(
        Method::GET,
        Url::parse(&format!(
            "https://wallhaven.cc/api/v1/search?q={terms}&page={page}"
        ))?,
    );
    let response = Client::new().execute(request).await?;
    if !response.status().is_success() {
        Err(anyhow!(
            "HTTP {} while attempting to communicate with WallHaven",
            response.status()
        ))?;
    }
    let response: BaseResponse = serde_json::from_slice(&response.bytes().await?)?;
    let wallpapers = response
        .data
        .into_iter()
        .map(|datum| {
            Wallpaper::new(
                datum.id.to_owned(),
                Some(datum.id),
                datum.path,
                Some(datum.url),
                source.to_string(),
            )
        })
        .filter(|wallpaper| !ids.contains(&wallpaper.id))
        .collect();
    Ok(wallpapers)
}
