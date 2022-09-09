use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::*;

fn gen_id() -> i32 {
    use rand::Rng;
    rand::thread_rng().gen_range(0..100000)
}

#[derive(Queryable, Serialize)]
pub struct Source {
    id: i32,
    subreddit: Option<String>,
}

#[derive(Insertable, Deserialize, Default)]
#[diesel(table_name = sources)]
pub struct NewSource {
    #[serde(default = "gen_id")]
    id: i32,
    subreddit: Option<String>,
}
