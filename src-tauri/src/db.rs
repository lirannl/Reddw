use crate::models::*;
use diesel::{delete, insert_into, prelude::*};
use tauri::AppHandle;

type Error = Box<dyn std::error::Error>;

trait ConnHolder {
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
    let t = insert_into(sources)
        .values(&source)
        .execute(&mut app.get_conn()?)
        .or_else(|e| Err(format!("{}", e.to_string())))?;
    Ok(())
}

#[tauri::command]
pub fn delete_source(app: AppHandle, doomed_id: i32) -> Result<(), String> {
    use crate::schema::sources::dsl::*;
    let t = delete(sources.filter(id.eq(doomed_id)))
        .execute(&mut app.get_conn()?)
        .or_else(|e| Err(format!("{}", e.to_string())))?;
    Ok(())
}
