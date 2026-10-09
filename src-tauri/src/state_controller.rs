// This file is for managing the application's state
use crate::config_controller;

use std::{path::PathBuf};
use std::sync::Mutex;
use core::error;

// AppState used for front-end - use for anything that will be modifying values
#[derive(Debug)]
pub struct AppState {
    data_dir: Mutex<std::path::PathBuf>,
}

// AppConfig to AppState
impl From<config_controller::AppConfig> for AppState {
    fn from(config: config_controller::AppConfig) -> Self {
        AppState { 
            data_dir: Mutex::new(config.data_path)
        }
    }
}

// Gets the data path that's currently being stored in the program's state
#[tauri::command]
pub fn get_data_path(state: tauri::State<'_, AppState>) -> Result<PathBuf, String> {
    Ok(state.data_dir.lock().unwrap().clone())
}

fn update_data_path_state (path: &PathBuf, state: tauri::State<'_, AppState>) -> Result<(), Box<dyn error::Error>>{
    let current_path = state.data_dir.lock().unwrap().clone();

    // We don't need to update anything if the path is the same
    if *path == current_path {
        return Ok(());
    }

    // Update the program's state
    *state.data_dir.lock().unwrap() = path.clone();
    Ok(())
}

// Updates the data path both in the program's state & config file
#[tauri::command]
pub fn update_data_path (
    path: String,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {

    // Save to program's state
    let new_path = PathBuf::from(path);
    if let Err(err) = update_data_path_state(&new_path, state) {
        println!("Error occured when updating program's data path: {}", err);
    }

    // Save to config file
    let config = config_controller::AppConfig {
        data_path: new_path.clone(),
    };
    config_controller::save_config(config, app).map_err(|e| e.to_string())
}