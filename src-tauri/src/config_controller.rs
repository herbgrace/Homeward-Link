use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use std::{fs, path::Path, error::Error};

const CONFIG_FILE: &str = "config.json";
const DEFAULT_BLOCK_SIZE: u16 = 600;
fn default_block_size() -> u16 { DEFAULT_BLOCK_SIZE }

use crate::state_controller;
// AppConfig only for saving/loading the json file - use for anything not modifying values
// Public values since it's used as an object for parameters/returning
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AppConfig {
    pub data_path: std::path::PathBuf,
    #[serde(default = "default_block_size")]
    pub loaded_block_size: u16
}

// AppState to AppConfig
// Used when saving the app's state to the config file
impl<'a> From<State<'_, state_controller::AppState>> for AppConfig {
    fn from(state: State<'_, state_controller::AppState>) -> Self {
        AppConfig { 
            data_path: state.data_dir.lock().unwrap().clone(),
            loaded_block_size: state.block_size.lock().unwrap().clone()
        }
    }
}

pub fn load_config(app: &mut tauri::App) -> Result<AppConfig, Box<dyn Error>> {
    let base_dir = app.path().app_local_data_dir().expect("Failed to get AppData Directory");
    load_config_from_dir(&base_dir)
}

// Reads the user's saved preferences on startup from the saved config file and stores them locally
pub(crate) fn load_config_from_dir(path: &Path) -> Result<AppConfig, Box<dyn std::error::Error>> {
    // Set up a default config in case it's the first time running
    let mut loaded = AppConfig {
        data_path: path.join("FlightData"),
        loaded_block_size: DEFAULT_BLOCK_SIZE
        // Add other config options here
    };

    // Ensure the AppData directory exists
    fs::create_dir_all(&path)?;
    let config_file = path.join(CONFIG_FILE);

    if config_file.exists() {
        if let Ok(cfg) = serde_json::from_str::<AppConfig>(&fs::read_to_string(&config_file)?) {
            loaded = cfg;
        }
    } else {
        write_config(&config_file, &loaded)?;
    }
    fs::create_dir_all(&loaded.data_path)?;
    Ok(loaded)
}

pub(crate) fn write_config(path: &Path, config: &AppConfig) -> Result<(), Box<dyn Error>> {
    fs::write(path, serde_json::to_string_pretty(config)?)?;
    Ok(())
}

pub fn save_config_from_state(app: tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from(app.state::<state_controller::AppState>());
    let path = app.path().app_local_data_dir().expect("Failed to get AppData Directory").join(CONFIG_FILE);
    write_config(&path, &config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::test::mock_app;
    use tempfile::tempdir;

    fn cfg(path: &Path, size: u16) -> AppConfig {
        AppConfig { data_path: path.to_path_buf(), loaded_block_size: size }
    }
    fn config_file(dir: &Path) -> std::path::PathBuf { dir.join(CONFIG_FILE) }

    #[test]
    fn first_run_creates_directories_and_default_config() {
        let tmp = tempdir().unwrap();
        let base = tmp.path().join("app"); // doesn't exist yet
        let loaded = load_config_from_dir(&base).unwrap();

        assert_eq!(loaded, cfg(&base.join("FlightData"), 600));
        assert!(base.join("FlightData").is_dir());

        let on_disk: AppConfig =
            serde_json::from_str(&fs::read_to_string(config_file(&base)).unwrap()).unwrap();
        assert_eq!(on_disk, loaded);
    }

    #[test]
    fn existing_config_is_loaded_as_is_and_not_rewritten() {
        let tmp = tempdir().unwrap();
        let custom = tmp.path().join("my").join("flights"); // nested + not created yet
        let json = serde_json::to_string(&cfg(&custom, 250)).unwrap();
        fs::write(config_file(tmp.path()), &json).unwrap();

        let loaded = load_config_from_dir(tmp.path()).unwrap();
        assert_eq!(loaded, cfg(&custom, 250));
        assert!(custom.is_dir(), "configured data dir should be created");
        assert!(!tmp.path().join("FlightData").exists(), "default dir should not be created");
        assert_eq!(fs::read_to_string(config_file(tmp.path())).unwrap(), json);
    }

    #[test]
    fn second_load_returns_the_same_config() {
        let tmp = tempdir().unwrap();
        let first = load_config_from_dir(tmp.path()).unwrap();
        let second = load_config_from_dir(tmp.path()).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn malformed_json_falls_back_to_defaults_without_touching_the_file() {
        for garbage in ["{ not json", "", "null", "[]", "42"] {
            let tmp = tempdir().unwrap();
            fs::write(config_file(tmp.path()), garbage).unwrap();

            let loaded = load_config_from_dir(tmp.path()).unwrap();
            assert_eq!(loaded, cfg(&tmp.path().join("FlightData"), 600), "input: {garbage:?}");
            assert_eq!(fs::read_to_string(config_file(tmp.path())).unwrap(), garbage);
        }
    }

    #[test]
    fn out_of_range_block_size_falls_back_to_defaults() {
        for bad in ["70000", "-1", "1.5", "\"600\""] {
            let tmp = tempdir().unwrap();
            let json = format!(r#"{{"data_path":"x","loaded_block_size":{bad}}}"#);
            fs::write(config_file(tmp.path()), json).unwrap();
            let loaded = load_config_from_dir(tmp.path()).unwrap();
            assert_eq!(loaded.loaded_block_size, 600, "block size {bad}");
            assert_eq!(loaded.data_path, tmp.path().join("FlightData"));
        }
    }

    // ⚠ needs #[serde(default = ...)]: an older config without block size keeps the user's path
    #[test]
    fn config_missing_block_size_keeps_the_saved_path() {
        let tmp = tempdir().unwrap();
        let custom = tmp.path().join("kept");
        fs::write(
            config_file(tmp.path()),
            serde_json::json!({ "data_path": custom }).to_string(),
        )
        .unwrap();

        let loaded = load_config_from_dir(tmp.path()).unwrap();
        assert_eq!(loaded, cfg(&custom, 600));
    }

    #[test]
    fn config_missing_data_path_falls_back_to_defaults() {
        let tmp = tempdir().unwrap();
        fs::write(config_file(tmp.path()), r#"{"loaded_block_size":100}"#).unwrap();
        let loaded = load_config_from_dir(tmp.path()).unwrap();
        assert_eq!(loaded, cfg(&tmp.path().join("FlightData"), 600));
    }

    #[test]
    fn unknown_fields_are_ignored() {
        let tmp = tempdir().unwrap();
        let custom = tmp.path().join("flights");
        let json = serde_json::json!({
            "data_path": custom, "loaded_block_size": 77, "future_option": true
        });
        fs::write(config_file(tmp.path()), json.to_string()).unwrap();
        assert_eq!(load_config_from_dir(tmp.path()).unwrap(), cfg(&custom, 77));
    }

    #[test]
    fn data_path_that_is_a_file_is_an_error() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("not_a_dir");
        fs::write(&file, "x").unwrap();
        write_config(&config_file(tmp.path()), &cfg(&file, 600)).unwrap();
        assert!(load_config_from_dir(tmp.path()).is_err());
    }

    #[test]
    fn base_dir_that_is_a_file_is_an_error() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("file");
        fs::write(&file, "x").unwrap();
        assert!(load_config_from_dir(&file).is_err());
    }

    #[test]
    fn write_config_round_trips_awkward_paths() {
        let tmp = tempdir().unwrap();
        let dir = tmp.path().join("flügel data").join("日本");
        fs::create_dir_all(&dir).unwrap();
        let original = cfg(&dir, u16::MAX);

        write_config(&config_file(tmp.path()), &original).unwrap();
        let back: AppConfig =
            serde_json::from_str(&fs::read_to_string(config_file(tmp.path())).unwrap()).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn write_config_into_missing_directory_is_an_error() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("missing").join(CONFIG_FILE);
        assert!(write_config(&path, &cfg(tmp.path(), 1)).is_err());
    }

    #[test]
    fn json_field_names_are_stable() {
        // The front end / existing config files depend on these exact keys
        let v = serde_json::to_value(cfg(Path::new("/x"), 600)).unwrap();
        assert!(v.get("data_path").is_some());
        assert_eq!(v["loaded_block_size"], 600);
        assert_eq!(v.as_object().unwrap().len(), 2);
    }

    #[test]
    fn app_state_converts_to_config_and_back() {
        let tmp = tempdir().unwrap();
        let app = mock_app();
        app.manage(state_controller::AppState::from(cfg(&tmp.path().join("flights"), 123)));
        let state = app.state::<state_controller::AppState>();

        *state.block_size.lock().unwrap() = 321; // simulate a settings change
        let snapshot = AppConfig::from(state);
        assert_eq!(snapshot, cfg(&tmp.path().join("flights"), 321));

        // persist, then reload as on the next app launch
        write_config(&config_file(tmp.path()), &snapshot).unwrap();
        assert_eq!(load_config_from_dir(tmp.path()).unwrap(), snapshot);
    }
}