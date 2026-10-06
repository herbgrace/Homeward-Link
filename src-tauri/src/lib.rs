use std::fs;
use serde::{Deserialize, Serialize};
use tauri::{Manager};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AppConfig {
    data_path: std::path::PathBuf
}

// AppState used for front-end
#[derive(Debug)]
struct AppState {
    data_dir: std::sync::Mutex<std::path::PathBuf>,
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(load_config)
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Reads the user's saved preferences from the saved config file and stores them locally 
fn load_config(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let default_path = app
                .path()
                .app_local_data_dir()
                .expect("Failed to get AppData Directory");

            let default_data_path = default_path.join("FlightData");

            let runtime_state = AppState {
                data_dir: std::sync::Mutex::new(default_data_path.clone())
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