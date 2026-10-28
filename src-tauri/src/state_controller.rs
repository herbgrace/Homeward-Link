use tauri::{Manager};

// This file is for managing the application's state
use crate::config_controller;

use std::{path::PathBuf};
use std::sync::Mutex;
use core::error;

// AppState used for front-end - use for anything that will be modifying values
#[derive(Debug)]
pub struct AppState {
    pub data_dir: Mutex<std::path::PathBuf>,
    current_block: Mutex<u16>,
    pub block_size: Mutex<u16>
}

// AppConfig to AppState
// Only used when booting the program & loading config file into state
impl From<config_controller::AppConfig> for AppState {
    fn from(config: config_controller::AppConfig) -> Self {
        AppState { 
            data_dir: Mutex::new(config.data_path),
            current_block: Mutex::new(0),
            block_size: Mutex::new(config.loaded_block_size)
        }
    }
}

// Gets the data path that's currently being stored in the program's state
#[tauri::command]
pub fn get_data_path(state: tauri::State<'_, AppState>) -> Result<PathBuf, String> {
    Ok(state.data_dir.lock().unwrap().clone())
}

fn update_data_path_state (path: &PathBuf, state: &tauri::State<'_, AppState>) -> Result<(), Box<dyn error::Error>>{
    let mut current_path = state.data_dir.lock().unwrap();
    if *path == *current_path {
        return Ok(());
    }
    *current_path = path.to_path_buf(); 

    Ok(())
}

// Updates the data path both in the program's state & config file
#[tauri::command]
pub fn update_data_path (path: String, app: tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    // Save to program's state
    let new_path = PathBuf::from(path);
    if let Err(err) = update_data_path_state(&new_path, &state) {
        return Err(err.to_string());
    }

    // Save to config file
    config_controller::save_config_from_state(app).map_err(|e| e.to_string())
}

fn update_block_size_state (new_size: u16, state: &tauri::State<'_, AppState>) -> Result<(), Box<dyn error::Error>>{
    let mut current_size = state.block_size.lock().unwrap();
    *current_size = new_size; 

    Ok(())
}

#[tauri::command]
pub fn update_block_size (new_size: u16, app: tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    // Save to program's state
    if let Err(err) = update_block_size_state(new_size, &state) {
        return Err(err.to_string());
    }

    // Save to config file
    config_controller::save_config_from_state(app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_block_size (state: tauri::State<'_, AppState>) -> Result<u16, String> {
    Ok(state.block_size.lock().unwrap().clone())
}

pub fn get_current_block(state: &tauri::State<'_, AppState>) -> Result<u16, String>{
    Ok(*state.current_block.lock().unwrap())
}

pub fn reset_current_block(state: &tauri::State<'_, AppState>) -> Result<(), String>{
    Ok(*state.current_block.lock().unwrap() = 0)
}

pub fn increment_current_block(state: &tauri::State<'_, AppState>) -> Result<(), String> {
    get_current_block(state)
        .and_then(|current| Ok(*state.current_block.lock().unwrap() = current + 1))
}

// pub fn get_receiver(state: &tauri::State<'_, AppState>) -> Result<Option<DeviceProfile>, String> {
//     Ok(*state.receiver_device.lock().unwrap().clone())
// }

// pub fn update_receiver(state: &tauri::State<'_, AppState>, profile: DeviceProfile) -> Result<(), Box<dyn error::Error>> {
//     let mut curr_profile = state.receiver_device.lock().unwrap();
//     *curr_profile = Some(profile); 

//     Ok(())  
// }