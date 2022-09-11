use std::{
    error::Error,
    fs::{create_dir, try_exists},
    io,
    path::PathBuf,
};

use entity::config;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set};
use tauri::{App, Manager};
use window_vibrancy::apply_mica;
#[cfg(target_os = "macos")]
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

fn ensure_dir_exists(path: &PathBuf) -> io::Result<()> {
    match try_exists(&path) {
        Ok(true) => Ok(()),
        Ok(false) => create_dir(&path),
        Err(e) => Err(e),
    }?;
    Ok(())
}

pub async fn reddw_setup(app: &mut App) -> Result<(), Box<dyn Error + Sync + Send>> {
    let config_folder = app
        .path_resolver()
        .app_dir()
        .ok_or("Failed to get config folder")?;
    ensure_dir_exists(&config_folder)?;

    let url = {
        let appdir = app.path_resolver().app_dir().ok_or("Invalid app dir")?;
        let url = format!(
            "sqlite://{}?mode=rwc",
            appdir.join("app.sqlite").to_str().unwrap()
        );
        url
    };

    let conn = Database::connect(&url).await?;

    Migrator::up(&conn, None).await?;
    
    // If config is empty, create a default one
    if config::Entity::find().all(&conn).await?.is_empty() {
        config::ActiveModel {
            allow_nsfw: Set(0),
            ..Default::default()
        }
        .insert(&conn)
        .await?;
    }

    app.manage::<DatabaseConnection>(conn);

    let window = app.get_window("main").unwrap();
    window.open_devtools();
    #[cfg(target_os = "macos")]
    apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow)
        .expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");

    #[cfg(target_os = "windows")]
    apply_mica(&window).expect("Unsupported platform! 'apply_blur' is only supported on Windows");

    // app.listen_global("change_wallpaper", |e| {});
    Ok(())
}
