use entity::{config, source};
use migration::DbErr;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, Set};
use tauri::{AppHandle, Manager};

fn get_connection(app: &AppHandle) -> Result<&DatabaseConnection, DbErr> {
    app.try_state::<DatabaseConnection>()
        .ok_or(DbErr::Conn("Failed to get connection".to_string()))
        .map(|conn| conn.inner())
}

#[tauri::command]
pub async fn get_sources(app: tauri::AppHandle) -> Result<Vec<source::Model>, String> {
    p_get_sources(app).await.or_else(|e| Err(e.to_string()))
}
async fn p_get_sources(app: tauri::AppHandle) -> Result<Vec<source::Model>, DbErr> {
    let conn = get_connection(&app)?;
    Ok(source::Entity::find().all(conn).await?)
}

#[tauri::command]
pub async fn add_source(app: tauri::AppHandle, source: source::Model) -> Result<(), String> {
    p_add_source(app, source)
        .await
        .or_else(|e| Err(e.to_string()))
}
async fn p_add_source(app: tauri::AppHandle, source: source::Model) -> Result<(), DbErr> {
    let conn = get_connection(&app)?;

    if vec![&source.subreddit]
        .into_iter()
        .map(|s| s.as_ref())
        .any(|s| match s {
            Some(s) => s.is_empty(),
            None => false,
        })
    {
        return Err(DbErr::Type("Missing value".to_string()));
    }
    source::ActiveModel {
        subreddit: Set(source.subreddit),
        ..Default::default()
    }
    .insert(conn)
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn remove_source(app: tauri::AppHandle, id: i32) -> Result<(), String> {
    p_remove_source(app, id)
        .await
        .or_else(|e| Err(e.to_string()))
}
async fn p_remove_source(app: tauri::AppHandle, id: i32) -> Result<(), DbErr> {
    let conn = get_connection(&app)?;
    let source = source::Entity::find_by_id(id)
        .one(conn)
        .await?
        .ok_or(DbErr::RecordNotFound("Not found.".to_string()))?;
    source.into_active_model().delete(conn).await?;
    Ok(())
}

#[tauri::command]
pub async fn get_config(app: tauri::AppHandle) -> Result<config::Model, String> {
    p_get_config(app).await.or_else(|e| Err(e.to_string()))
}
async fn p_get_config(app: tauri::AppHandle) -> Result<config::Model, DbErr> {
    let conn = get_connection(&app)?;
    let config = config::Entity::find()
        .one(conn)
        .await?
        .ok_or(DbErr::RecordNotFound("No config found".to_string()))?;
    Ok(config)
}

#[tauri::command]
pub async fn update_config(app: tauri::AppHandle, update: config::Model) -> Result<(), String> {
    p_update_config(app, update)
        .await
        .or_else(|e| Err(e.to_string()))
}
async fn p_update_config(app: tauri::AppHandle, update: config::Model) -> Result<(), DbErr> {
    let conn = get_connection(&app)?;
    let mut config = config::Entity::find()
        .one(conn)
        .await?
        .ok_or(DbErr::RecordNotFound("No config found".to_string()))?
        .into_active_model();

    config.allow_nsfw = Set(update.allow_nsfw);
    
    config.update(conn).await?;
    Ok(())
}
