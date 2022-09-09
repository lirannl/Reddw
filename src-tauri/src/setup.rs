use std::{error::Error, fs::{create_dir, try_exists}, io, path::PathBuf};

use tauri::{api::path::app_dir, App};

fn ensure_dir_exists(path: &PathBuf) -> io::Result<()> {
    match try_exists(&path) {
        Ok(true) => Ok(()),
        Ok(false) => create_dir(&path),
        Err(e) => Err(e),
    }?;
    Ok(())
}

fn ensure_db_exists(path: &PathBuf) -> io::Result<()> {
    let db_path = path.join("config.db");
    match try_exists(&db_path) {
        Ok(true) => Ok(()),
        Ok(false) => {
            
            Ok(())
        }
        Err(e) => Err(e),
    }?;
    Ok(())
}

pub fn reddw_setup(app: &mut App) -> Result<(), Box<dyn Error>> {
    let config_folder = app_dir(app.config().as_ref())
        .ok_or("Error getting config folder")?;
    ensure_dir_exists(&config_folder)?;
    ensure_db_exists(&config_folder)?;
    Ok(())
}
