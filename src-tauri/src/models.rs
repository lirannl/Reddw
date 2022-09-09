use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::schema::*;

fn gen_id() -> i32 {
    use rand::Rng;
    rand::thread_rng().gen_range(0..100000)
}

#[derive(Queryable, Serialize, TS)]
#[ts(export)]
pub struct Source {
    id: i32,
    subreddit: Option<String>,
}

#[derive(Insertable, Deserialize, TS)]
#[diesel(table_name = sources)]
#[ts(export)]
pub struct NewSource {
    #[serde(default = "gen_id")]
    id: i32,
    subreddit: String,
}

#[derive(Queryable, Insertable, Serialize, Deserialize, TS)]
#[ts(export)]
#[diesel(table_name = config)]
pub struct Config {
    id: i32,
    refresh_minutes: i32,
    allow_nsfw: i32
}

impl Default for Config {
    fn default() -> Self {
        Config {
            id: 0,
            refresh_minutes: 60,
            allow_nsfw: 0
        }
    }
}
