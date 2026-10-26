mod state_controller;
mod config_controller;
mod file_controller;
mod usb_reader;
mod telem_decoder;

use core::error;
use tauri::{Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(setup_program)
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            state_controller::update_data_path,
            state_controller::get_data_path,
            state_controller::update_block_size,
            state_controller::get_block_size,
            file_controller::get_all_saved_flights,
            file_controller::load_all_flight_data,
            file_controller::load_flight_block,
            usb_reader::start_serial,
            usb_reader::stop_serial
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_program(app: &mut tauri::App) -> Result<(), Box<dyn error::Error>> {
    // Load config into app's state
    let loaded_config = config_controller::load_config(app).unwrap();
    let config_state = state_controller::AppState::from(loaded_config);
    app.manage(config_state);
    app.manage(usb_reader::SerialManager::default());

    Ok(())
}