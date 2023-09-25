#![feature(
    async_closure,
    async_fn_in_trait)]

use reddw_source_plugin::{SourcePlugin, Wallpaper};

pub struct WallHaven {}

impl SourcePlugin for WallHaven {
    async fn get_wallpapers(&self) -> Vec<reddw_source_plugin::Wallpaper> {
        vec![Wallpaper::new("example".to_string(), "name".to_string(), "".to_string(), Some("".to_string()), "".to_string())]
    }
}