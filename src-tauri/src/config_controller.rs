use serde::{Deserialize, Serialize};
use tauri::Manager;
use std::{fs};
// AppConfig only for saving/loading the json file - use for anything not modifying values
// Public values since it's used as an object for parameters/returning
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub data_path: std::path::PathBuf,
}

// Reads the user's saved preferences on startup from the saved config file and stores them locally
pub fn load_config(app: &mut tauri::App) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let default_path = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get AppData Directory");

    let default_data_path = default_path.join("FlightData");

    // Set up a default config in case it's the first time running
    let mut loaded_config = AppConfig {
        data_path: default_data_path.clone(),
        // Add other config options here
    };

    // Ensure the AppData directory exists
    fs::create_dir_all(&default_path)?;
    let config_file_path = default_path.join("config.json");

    if config_file_path.exists() {
        // Load saved configs
        let config_data = fs::read_to_string(&config_file_path)?;
        if let Ok(config) = serde_json::from_str::<AppConfig>(&config_data) {
            loaded_config = config;
        }
    } else {
        // Config file doesn't exist - Create a default one
        let json = serde_json::to_string_pretty(&loaded_config)?;
        fs::write(&config_file_path, json)?;
    }

    // Ensure the active data directory exists
    let active_data_dir = loaded_config.data_path.clone();
    fs::create_dir_all(&active_data_dir)?;

    Ok(loaded_config.clone())
}

pub fn save_config(config: AppConfig, app: tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
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