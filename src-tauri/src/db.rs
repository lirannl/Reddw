use tauri::Runtime;

#[tauri::command]
pub async fn check_config<R: Runtime>(
    app: tauri::AppHandle<R>,
    window: tauri::Window<R>,
) -> Result<(), String> {
    let config_folder = app
        .path_resolver()
        .app_dir()
        .ok_or("Error getting config folder")?
        .join("config.db");
    Ok(())
}
