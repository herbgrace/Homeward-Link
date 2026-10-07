use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::Manager;

// AppConfig only for saving/loading the json file - use for anything not modifying values
#[derive(Serialize, Deserialize, Clone, Debug)]
struct AppConfig {
    data_path: std::path::PathBuf,
}

// AppState used for front-end - use for anything that will be modifying values
#[derive(Debug)]
struct AppState {
    data_dir: std::sync::Mutex<std::path::PathBuf>,
}

#[tauri::command]
async fn get_data_path(state: tauri::State<'_, AppState>) -> Result<PathBuf, String> {
    Ok(state.data_dir.lock().unwrap().clone())
}

#[tauri::command]
fn update_data_path(
    path: String,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let new_path = PathBuf::from(path);
    let current_path = state.data_dir.lock().unwrap().clone();

    // We don't need to update anything if the path is the same
    if new_path == current_path {
        return Ok(());
    }

    // Update the program's state
    *state.data_dir.lock().unwrap() = new_path.clone();

    // Save that jawn
    let config = AppConfig {
        data_path: new_path.clone(),
    };
    // println!("{:?}", config);

    save_config(config, app).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(load_config)
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            update_data_path,
            get_data_path
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn save_config(config: AppConfig, app: tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // Get the config file location
    let config_path = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get AppData Directory")
        .join("config.json");

    // Save the config based on passed in parameter
    let json = serde_json::to_string_pretty(&config)?;
    fs::write(&config_path, json)?;

    Ok(())
}

// Reads the user's saved preferences on startup from the saved config file and stores them locally
fn load_config(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let default_path = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get AppData Directory");

    let default_data_path = default_path.join("FlightData");

    let runtime_state = AppState {
        data_dir: std::sync::Mutex::new(default_data_path.clone()),
    };

    // Ensure the default directory exists
    fs::create_dir_all(&default_path)?;
    let config_file_path = default_path.join("config.json");

    if config_file_path.exists() {
        // Load saved configs
        let config_data = fs::read_to_string(&config_file_path)?;
        if let Ok(config) = serde_json::from_str::<AppConfig>(&config_data) {
            *runtime_state.data_dir.lock().unwrap() = config.data_path;
            // Add other configs here
        }
    } else {
        // Config file doesn't exist - Create a default one
        let default_config = AppConfig {
            data_path: default_data_path.clone(),
            // Add other default configs here
        };
        let json = serde_json::to_string_pretty(&default_config)?;
        fs::write(&config_file_path, json)?;
    }

    // Ensure the active data directory exists
    let active_data_dir = runtime_state.data_dir.lock().unwrap().clone();
    fs::create_dir_all(&active_data_dir)?;

    // Save the loaded configs into Tauri's runtime
    println!("{:?}", runtime_state);
    app.manage(runtime_state);

    Ok(())
}
