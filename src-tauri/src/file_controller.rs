use csv::StringRecord;

use crate::state_controller;

use std::{fs};

const BLOCK_SIZE: u16 = 600; // At most 10 entries/sec, so 60 seconds per refresh

#[tauri::command]
pub async fn get_all_saved_flights(state: tauri::State<'_, state_controller::AppState>) -> Result<Vec<String>, String> {
    let supported_extensions = ["csv", "telem", "eeprom"];

    // Reset the current block since user isn't looking at a specific flight
    state_controller::reset_current_block(&state)?;

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
        let mut extension: &str = ""; // Assume it's a folder by default

        // If the extension exists (And the file name isn't empty), extract the extension
        if let Some(e) = file_path.extension() {
            extension = e.to_str().unwrap();
        }

        // Only return the file if the extension is supported
        if supported_extensions.contains(&extension) {
            if let Some(s) = file_path.to_str() {
                files.push(s.to_string());
            }
        }
        
    }
    return Ok(files);
}

// Read all the data in a given CSV file
// Unusued for now, but could be useful in the future
#[tauri::command]
pub async fn load_all_flight_data(file_path: String) -> Result<Vec<Vec<String>>, String> {
    let reader = csv::ReaderBuilder::new()
        .has_headers(false) // Allows inclusion of headers in result without additional logic
        .from_path(file_path)
        .map_err(|e| e.to_string());
    let mut records: Vec<Vec<String>> = Vec::new();

    for result in reader?.records() {
        let record = result.unwrap();
        let row: Vec<String> = record.iter().map(|field| field.to_string()).collect();
        records.push(row);
    }

    Ok(records)
}

// u16 allows for max flight lengths of ~76 days, which should be plenty
// u8 allows for max lengths of ~7 hours, which could theoretically cause problems
#[tauri::command]
pub async fn load_flight_block(state: tauri::State<'_, state_controller::AppState>, file_path: String) -> Result<Vec<Vec<String>>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .from_path(file_path)
        .map_err(|e| e.to_string())?;

    let block = state_controller::get_current_block(&state)?;
    let skipped_lines: u32 = u32::from(block) * u32::from(BLOCK_SIZE);

    let mut records: Vec<Vec<String>> = Vec::new();
    let mut line: StringRecord = StringRecord::new();

    // Always add headers as the first entry
    let headers_record = reader.headers().map_err(|e| e.to_string())?;
    let headers: Vec<String> = headers_record.iter().map(|field| field.to_string()).collect();
    records.push(headers);

    // Skip to the block that we care about
    for _ in 0..skipped_lines {
        if !reader.read_record(&mut line).map_err(|e| e.to_string())? {
            return Ok(records);
        }
    }

    // Load the entire block
    for _ in 0..BLOCK_SIZE {
        if reader.read_record(&mut line).map_err(|e| e.to_string())? {
            let row: Vec<String> = line.iter().map(|field| field.to_string()).collect();
            records.push(row);
        }
    }

    // Increment the state for future calls, then return loaded lines
    state_controller::increment_current_block(&state)?;
    Ok(records)
}