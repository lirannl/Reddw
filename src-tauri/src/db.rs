use std::error::Error;

use crate::models::*;
use diesel::{delete, insert_into, prelude::*};
use tauri::{App, AppHandle};

pub trait ConnHolder {
    fn get_conn(&self) -> Result<SqliteConnection, String>;
}

impl ConnHolder for AppHandle {
    fn get_conn(&self) -> Result<SqliteConnection, String> {
        SqliteConnection::establish(
            format!(
                "sqlite://{}?mode=rwc",
                self.path_resolver()
                    .app_dir()
                    .unwrap()
                    .join("config.sqlite")
                    .to_str()
                    .unwrap()
            )
            .as_str(),
        )
        .or(Err("Failed to connect to database".into()))
    }
}

#[tauri::command]
pub fn get_sources(app: AppHandle) -> Result<Vec<Source>, String> {
    use crate::schema::sources::dsl::*;
    let res = sources
        .load::<Source>(&mut app.get_conn()?)
        .or_else(|e| Err(format!("{:?}", e)))?;
    Ok(res)
}

#[tauri::command]
pub fn add_source(app: AppHandle, source: NewSource) -> Result<(), String> {
    use crate::schema::sources::dsl::*;
    insert_into(sources)
        .values(&source)
        .execute(&mut app.get_conn()?)
        .or_else(|e| Err(format!("{}", e.to_string())))?;
    Ok(())
}

#[tauri::command]
pub fn delete_source(app: AppHandle, doomed_id: i32) -> Result<(), String> {
    use crate::schema::sources::dsl::*;
    delete(sources.filter(id.eq(doomed_id)))
        .execute(&mut app.get_conn()?)
        .or_else(|e| Err(format!("{}", e.to_string())))?;
    Ok(())
}

pub fn populate_config(app: AppHandle) -> Result<(), Box<dyn Error + Send + Sync>> {
    use crate::schema::config::dsl::*;
    let mut conn = app.get_conn()?;
    if (config.load::<Config>(&mut conn).or_else(|err| {
        Err(err)})?.len() == 0) {
        insert_into(config).values(Config::default()).execute(&mut conn)?;
    }
    Ok(())
}
