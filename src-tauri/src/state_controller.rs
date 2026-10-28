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
    let mut current = state.current_block.lock().unwrap();
    *current = current.checked_add(1).ok_or("Block Counter Overflow")?;

    Ok(())
}

// pub fn get_receiver(state: &tauri::State<'_, AppState>) -> Result<Option<DeviceProfile>, String> {
//     Ok(*state.receiver_device.lock().unwrap().clone())
// }

// pub fn update_receiver(state: &tauri::State<'_, AppState>, profile: DeviceProfile) -> Result<(), Box<dyn error::Error>> {
//     let mut curr_profile = state.receiver_device.lock().unwrap();
//     *curr_profile = Some(profile); 

//     Ok(())  
// }

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::test::{mock_app, MockRuntime};

    fn app_with(path: &str, block_size: u16) -> tauri::App<MockRuntime> {
        let app = mock_app();
        app.manage(AppState::from(config_controller::AppConfig {
            data_path: PathBuf::from(path),
            loaded_block_size: block_size,
        }));
        app
    }

    #[test]
    fn from_config_copies_fields_and_starts_at_block_zero() {
        let app = app_with("/data/flights", 250);
        let state = app.state::<AppState>();
        assert_eq!(get_data_path(state.clone()).unwrap(), PathBuf::from("/data/flights"));
        assert_eq!(get_block_size(state.clone()).unwrap(), 250);
        assert_eq!(get_current_block(&state).unwrap(), 0);
    }

    #[test]
    fn update_data_path_state_replaces_the_path() {
        let app = app_with("/old", 600);
        let state = app.state::<AppState>();
        update_data_path_state(&PathBuf::from("/new"), &state).unwrap();
        assert_eq!(get_data_path(state).unwrap(), PathBuf::from("/new"));
    }

    #[test]
    fn update_data_path_state_same_path_is_a_noop() {
        let app = app_with("/same", 600);
        let state = app.state::<AppState>();
        update_data_path_state(&PathBuf::from("/same"), &state).unwrap();
        assert_eq!(get_data_path(state).unwrap(), PathBuf::from("/same"));
    }

    #[test]
    fn update_data_path_state_does_not_validate() {
        // Documents current behavior: any string is accepted here (even empty).
        // Bad paths only surface later, in get_all_saved_flights.
        let app = app_with("/old", 600);
        let state = app.state::<AppState>();
        update_data_path_state(&PathBuf::from(""), &state).unwrap();
        assert_eq!(get_data_path(state).unwrap(), PathBuf::from(""));
    }

    #[test]
    fn update_block_size_state_accepts_the_full_u16_range() {
        let app = app_with("/d", 600);
        let state = app.state::<AppState>();
        for size in [0u16, 1, 600, u16::MAX] {
            update_block_size_state(size, &state).unwrap();
            assert_eq!(get_block_size(state.clone()).unwrap(), size);
        }
    }

    #[test]
    fn block_counter_increments_and_resets() {
        let app = app_with("/d", 600);
        let state = app.state::<AppState>();
        for expected in 1..=5u16 {
            increment_current_block(&state).unwrap();
            assert_eq!(get_current_block(&state).unwrap(), expected);
        }
        reset_current_block(&state).unwrap();
        assert_eq!(get_current_block(&state).unwrap(), 0);
        reset_current_block(&state).unwrap(); // resetting at zero is fine
        assert_eq!(get_current_block(&state).unwrap(), 0);
    }

    #[test]
    fn changing_block_size_does_not_touch_the_counter() {
        let app = app_with("/d", 600);
        let state = app.state::<AppState>();
        increment_current_block(&state).unwrap();
        update_block_size_state(10, &state).unwrap();
        assert_eq!(get_current_block(&state).unwrap(), 1);
    }

    #[test]
    fn all_state_handles_see_the_same_data() {
        let app = app_with("/d", 600);
        let a = app.state::<AppState>();
        let b = app.state::<AppState>();
        increment_current_block(&a).unwrap();
        update_block_size_state(42, &a).unwrap();
        assert_eq!(get_current_block(&b).unwrap(), 1);
        assert_eq!(get_block_size(b).unwrap(), 42);
    }

    #[test]
    fn increment_at_u16_max_errors_instead_of_wrapping() {
        let app = app_with("/d", 600);
        let state = app.state::<AppState>();
        *state.current_block.lock().unwrap() = u16::MAX;
        assert!(increment_current_block(&state).is_err());
        assert_eq!(get_current_block(&state).unwrap(), u16::MAX);
    }

    #[test]
    fn concurrent_increments_are_not_lost() {
        let app = app_with("/d", 600);
        let state = app.state::<AppState>();
        std::thread::scope(|s| {
            for _ in 0..8 {
                s.spawn(|| {
                    for _ in 0..500 {
                        increment_current_block(&state).unwrap();
                    }
                });
            }
        });
        assert_eq!(get_current_block(&state).unwrap(), 4000);
    }
}