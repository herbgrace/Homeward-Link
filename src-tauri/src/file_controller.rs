use crate::state_controller;

use std::fs;

#[tauri::command]
pub async fn get_all_saved_flights(state: tauri::State<'_, state_controller::AppState>) -> Result<Vec<String>, String> {
    // Get file path from app state
    let directory_path = state_controller::get_data_path(state)?;
    if !directory_path.is_dir() {
        return Err("Saved data path is not a valid directory".to_string());
    }

    let entries = fs::read_dir(directory_path)
        .map_err(|err| err.to_string())?;

    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| err.to_string())?;
        let file_path = entry.path();
        if file_path.is_file() {
            if let Some(s) = file_path.to_str() {
                files.push(s.to_string());
            }
        }
    }
    return Ok(files);
}

// TODO - get data from a single csv file
// Load in "blocks" of ~1000 lines(?)
// Front-end shouldn't keep track of which block, but how to continuously send blocks to front-end?
// #[tauri::command]
// async fn get_saved_flight_block() {
    
// }