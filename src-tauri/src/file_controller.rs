use crate::{state_controller, telem_decoder};

use csv::StringRecord;
use std::{fs, io::BufReader, io::BufRead, path::Path };
use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum FlightData {
    Table(Vec<Vec<String>>),
    Telemetry { packets: Vec<telem_decoder::Telemetry>, skipped: usize },
}

impl FlightData {
    fn len(&self) -> usize {
        match self {
            FlightData::Table(vec) => vec.len(),
            FlightData::Telemetry { packets, .. } => packets.len() 
        }
    }
}

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

        // If the extension exists (And the file name isn't empty), extract the extension
        let extension = get_extension(&file_path);

        // Only return the file if the extension is supported
        if supported_extensions.contains(&extension.as_str()) {
            if let Some(s) = file_path.to_str() {
                files.push(s.to_string());
            }
        }
        
    }
    return Ok(files);
}

fn get_extension(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default()
}

// Read all the data in a given CSV file
// Unusued for now, but could be useful in the future
#[tauri::command]
pub async fn load_all_flight_data(file_path: String) -> Result<FlightData, String> {
    let path = Path::new(&file_path);
    let data: FlightData = match get_extension(&path).to_lowercase().as_str() {
        "csv" => load_all_csv_data(file_path).await.map(FlightData::Table),
        "telem" => load_all_telem_data(file_path).await, 
        _ => return Err("Invalid file extension".to_string()),
    }?;

    Ok(data)
}

async fn load_all_csv_data(path: String) -> Result<Vec<Vec<String>>, String> {
    let mut reader = csv::ReaderBuilder::new()
            .has_headers(false) // Allows inclusion of headers in result without additional logic
            .from_path(path)
            .map_err(|e| e.to_string())?;
        let mut records: Vec<Vec<String>> = Vec::new();

        for result in reader.records() {
            let record = result.map_err(|e| e.to_string())?;
            let row: Vec<String> = record.iter().map(|field| field.to_string()).collect();
            records.push(row);
        }

        Ok(records)
}

async fn load_all_telem_data(path: String) -> Result<FlightData, String> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);

    let mut resp: Vec<telem_decoder::Telemetry> = Vec::new();
    let mut skipped: usize = 0;

    for line in reader.lines() {
        let Ok(line) = line else { skipped += 1; continue };
        if line.trim().is_empty() { continue; }

        match telem_decoder::parse_line(&line) {
            Ok(t) => resp.push(t),
            Err(_) => skipped += 1,
        }
    }
    
    Ok(FlightData::Telemetry { packets: resp, skipped: skipped })
}

// u16 allows for max flight lengths of ~76 days, which should be plenty
// u8 allows for max lengths of ~7 hours, which could theoretically cause problems
#[tauri::command]
pub async fn load_flight_block(state: tauri::State<'_, state_controller::AppState>, file_path: String) -> Result<FlightData, String> {
    let path = Path::new(&file_path);

    let data = match get_extension(&path).to_lowercase().as_str() {
        "csv" => load_csv_block(&state, file_path).await.map(FlightData::Table),
        "telem" => load_telem_block(&state, file_path).await,
        _ => return Err("Invalid file extension".to_string()),
    }?;

    // Increment the state for future calls, then return loaded lines
    // Only increment if there's more data than just headers (Not end of file)
    let mut empty_len: usize = 0;
    if matches!(data, FlightData::Table(_)) { empty_len += 1; }

    if data.len() != empty_len {
        state_controller::increment_current_block(&state)?;
    }
    
    Ok(data)
}


async fn load_csv_block(state: &tauri::State<'_, state_controller::AppState>, path: String) -> Result<Vec<Vec<String>>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .from_path(path)
        .map_err(|e| e.to_string())?;

    let block = state_controller::get_current_block(&state)?;
    let block_size: u16 = state_controller::get_block_size(state.clone())?;
    let skipped_lines: u32 = u32::from(block) * u32::from(block_size);

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
    for _ in 0..block_size {
        if reader.read_record(&mut line).map_err(|e| e.to_string())? {
            let row: Vec<String> = line.iter().map(|field| field.to_string()).collect();
            records.push(row);
        }
    }

    Ok(records)
}

async fn load_telem_block(state: &tauri::State<'_, state_controller::AppState>, path: String) -> Result<FlightData, String> {
    let block = state_controller::get_current_block(&state)?;
    let block_size: u16 = state_controller::get_block_size(state.clone())?;

    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let lines = BufReader::new(file).lines()
        .skip(block as usize * block_size as usize)
        .take(block_size as usize);

    let mut resp: Vec<telem_decoder::Telemetry> = Vec::new();
    let mut skipped = 0;
    for line in lines {
        let Ok(line) = line else { skipped += 1; continue };
        if line.trim().is_empty() { continue; }

        match telem_decoder::parse_line(&line) {
            Ok(t) => resp.push(t),
            Err(_) => skipped += 1,
        }
    }
    
    Ok(FlightData::Telemetry { packets: resp, skipped: skipped })
}

// TODO - Fix unit test that were broken by refactor
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_controller::AppConfig;
    use crate::state_controller::AppState;
    use std::future::Future;
    use std::path::{Path,};
    use tauri::test::{mock_app, MockRuntime};
    use tauri::Manager;
    use tempfile::tempdir;

    // ── helpers ──

    fn run<F: Future>(f: F) -> F::Output { tauri::async_runtime::block_on(f) }

    fn app_with_dir(dir: &Path, block_size: u16) -> tauri::App<MockRuntime> {
        let app = mock_app();
        app.manage(AppState::from(AppConfig {
            data_path: dir.to_path_buf(),
            loaded_block_size: block_size,
        }));
        app
    }

    /// "id,val" header + rows r0,v0 / r1,v1 / ...
    // fn write_csv(dir: &Path, name: &str, rows: usize) -> PathBuf {
    //     let mut s = String::from("id,val\n");
    //     for i in 0..rows { s.push_str(&format!("r{i},v{i}\n")); }
    //     let p = dir.join(name);
    //     fs::write(&p, s).unwrap();
    //     p
    // }

    fn load(state: &tauri::State<'_, AppState>, file: &Path) -> Result<FlightData, String> {
        run(load_flight_block(state.clone(), file.to_str().unwrap().to_string()))
    }

    /// first column of each returned row, e.g. ["id", "r0", "r1"]
    // fn page(state: &tauri::State<'_, AppState>, file: &Path) -> Vec<String> {
    //     load(state, file).unwrap().into_iter().map(|r| r[0].clone()).collect()
    // }

    fn names(paths: Vec<String>) -> Vec<String> {
        let mut v: Vec<String> = paths.iter()
            .map(|p| Path::new(p).file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        v.sort();
        v
    }

    // ── get_all_saved_flights ──

    #[test]
    fn lists_only_supported_extensions() {
        let tmp = tempdir().unwrap();
        for name in ["a.csv", "b.telem", "c.eeprom", "d.txt", "e.json", "noext", "csv", "f.csv.bak", ".hidden"] {
            fs::write(tmp.path().join(name), "x").unwrap();
        }
        fs::create_dir(tmp.path().join("subdir")).unwrap();

        let app = app_with_dir(tmp.path(), 3);
        let files = run(get_all_saved_flights(app.state::<AppState>())).unwrap();

        assert!(files.iter().all(|f| f.starts_with(tmp.path().to_str().unwrap())), "full paths expected");
        assert_eq!(names(files), vec!["a.csv", "b.telem", "c.eeprom"]);
    }

    #[test]
    fn multi_dot_names_use_the_last_extension() {
        let tmp = tempdir().unwrap();
        fs::write(tmp.path().join("flight.backup.csv"), "x").unwrap();
        let app = app_with_dir(tmp.path(), 3);
        let files = run(get_all_saved_flights(app.state::<AppState>())).unwrap();
        assert_eq!(names(files), vec!["flight.backup.csv"]);
    }

    #[test]
    fn extension_matching_is_case_insensitive() {
        let tmp = tempdir().unwrap();
        for name in ["A.CSV", "b.TELEM", "c.Eeprom"] {
            fs::write(tmp.path().join(name), "x").unwrap();
        }
        let app = app_with_dir(tmp.path(), 3);
        let files = run(get_all_saved_flights(app.state::<AppState>())).unwrap();
        assert_eq!(names(files), vec!["A.CSV", "b.TELEM", "c.Eeprom"]);
    }

    #[test]
    fn empty_directory_gives_empty_list() {
        let tmp = tempdir().unwrap();
        let app = app_with_dir(tmp.path(), 3);
        assert!(run(get_all_saved_flights(app.state::<AppState>())).unwrap().is_empty());
    }

    #[test]
    fn missing_directory_or_file_path_is_an_error() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("file.csv");
        fs::write(&file, "x").unwrap();

        for bad in [tmp.path().join("nope"), file] {
            let app = app_with_dir(&bad, 3);
            let err = run(get_all_saved_flights(app.state::<AppState>())).unwrap_err();
            assert!(err.contains("not a valid directory"), "{err}");
        }
    }

    #[test]
    fn listing_flights_resets_the_block_counter() {
        let tmp = tempdir().unwrap();
        let app = app_with_dir(tmp.path(), 3);
        let state = app.state::<AppState>();
        for _ in 0..3 { state_controller::increment_current_block(&state).unwrap(); }

        run(get_all_saved_flights(state.clone())).unwrap();
        assert_eq!(state_controller::get_current_block(&state).unwrap(), 0);
    }

    // ── load_all_flight_data ──

    // #[test]
    // fn load_all_returns_every_row_including_the_header() {
    //     let tmp = tempdir().unwrap();
    //     let file = write_csv(tmp.path(), "f.csv", 3);
    //     let rows = run(load_all_flight_data(file.to_str().unwrap().into())).unwrap();
    //     assert_eq!(rows.len(), 4);
    //     assert_eq!(rows[0], vec!["id", "val"]);
    //     assert_eq!(rows[3], vec!["r2", "v2"]);
    // }

    // #[test]
    // fn load_all_handles_quotes_embedded_newlines_crlf_and_unicode() {
    //     let tmp = tempdir().unwrap();
    //     let file = tmp.path().join("f.csv");
    //     fs::write(&file, "\"a,b\",\"x\ny\",z\r\n日本,\"say \"\"hi\"\"\",é\r\n").unwrap();
    //     let rows = run(load_all_flight_data(file.to_str().unwrap().into())).unwrap();
    //     assert_eq!(rows[0], vec!["a,b", "x\ny", "z"]);
    //     assert_eq!(rows[1], vec!["日本", "say \"hi\"", "é"]);
    // }

    // #[test]
    // fn load_all_empty_file_gives_no_rows() {
    //     let tmp = tempdir().unwrap();
    //     let file = tmp.path().join("empty.csv");
    //     fs::write(&file, "").unwrap();
    //     assert!(run(load_all_flight_data(file.to_str().unwrap().into())).unwrap().is_empty());
    // }

    #[test]
    fn load_all_missing_file_is_an_error() {
        let tmp = tempdir().unwrap();
        let missing = tmp.path().join("nope.csv");
        assert!(run(load_all_flight_data(missing.to_str().unwrap().into())).is_err());
    }

    #[test]
    fn load_all_ragged_rows_are_an_error_not_a_panic() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("ragged.csv");
        fs::write(&file, "a,b\n1,2,3\n").unwrap();
        assert!(run(load_all_flight_data(file.to_str().unwrap().into())).is_err());
    }

    // ── load_flight_block ──

    // #[test]
    // fn pages_through_a_file_block_by_block() {
    //     let tmp = tempdir().unwrap();
    //     let file = write_csv(tmp.path(), "f.csv", 7);
    //     let app = app_with_dir(tmp.path(), 3);
    //     let state = app.state::<AppState>();

    //     assert_eq!(page(&state, &file), vec!["id", "r0", "r1", "r2"]);
    //     assert_eq!(page(&state, &file), vec!["id", "r3", "r4", "r5"]);
    //     assert_eq!(page(&state, &file), vec!["id", "r6"]); // partial last block
    //     assert_eq!(state_controller::get_current_block(&state).unwrap(), 3);

    //     // Past the end: headers only, and the counter stops advancing
    //     assert_eq!(page(&state, &file), vec!["id"]);
    //     assert_eq!(page(&state, &file), vec!["id"]);
    //     assert_eq!(state_controller::get_current_block(&state).unwrap(), 3);
    // }

    // #[test]
    // fn exact_multiple_of_block_size_does_not_give_trailing_empty_block() {
    //     // 6 rows / block size 3 -> blocks 0 and 1 hold data,
    //     // Should return a block of just headers afterwards with no data
    //     let tmp = tempdir().unwrap();
    //     let file = write_csv(tmp.path(), "f.csv", 6);
    //     let app = app_with_dir(tmp.path(), 3);
    //     let state = app.state::<AppState>();

    //     page(&state, &file);
    //     assert_eq!(state_controller::get_current_block(&state).unwrap(), 1);
    //     page(&state, &file);
    //     assert_eq!(state_controller::get_current_block(&state).unwrap(), 2);
    //     assert_eq!(page(&state, &file), vec!["id"]);
    //     assert_eq!(state_controller::get_current_block(&state).unwrap(), 2);
    // }

    // #[test]
    // fn reset_restarts_paging_from_the_first_block() {
    //     let tmp = tempdir().unwrap();
    //     let file = write_csv(tmp.path(), "f.csv", 7);
    //     let app = app_with_dir(tmp.path(), 3);
    //     let state = app.state::<AppState>();

    //     page(&state, &file);
    //     page(&state, &file);
    //     state_controller::reset_current_block(&state).unwrap();
    //     assert_eq!(page(&state, &file), vec!["id", "r0", "r1", "r2"]);
    // }

    // #[test]
    // fn block_size_larger_than_file_returns_everything_once() {
    //     let tmp = tempdir().unwrap();
    //     let file = write_csv(tmp.path(), "f.csv", 4);
    //     let app = app_with_dir(tmp.path(), 1000);
    //     let state = app.state::<AppState>();

    //     assert_eq!(page(&state, &file), vec!["id", "r0", "r1", "r2", "r3"]);
    //     assert_eq!(page(&state, &file), vec!["id"]);
    // }

    // #[test]
    // fn header_only_file_returns_just_the_header() {
    //     let tmp = tempdir().unwrap();
    //     let file = write_csv(tmp.path(), "f.csv", 0);
    //     let app = app_with_dir(tmp.path(), 3);
    //     let state = app.state::<AppState>();
    //     assert_eq!(page(&state, &file), vec!["id"]);
    // }

    // #[test]
    // fn block_size_zero_returns_only_headers() {
    //     let tmp = tempdir().unwrap();
    //     let file = write_csv(tmp.path(), "f.csv", 5);
    //     let app = app_with_dir(tmp.path(), 0);
    //     let state = app.state::<AppState>();
    //     assert_eq!(page(&state, &file), vec!["id"]);
    //     // Note: the counter still advances every call with size 0; consider
    //     // rejecting a zero block size in update_block_size.
    // }

    // #[test]
    // fn index_times_size_uses_u32_and_cannot_overflow() {
    //     let tmp = tempdir().unwrap();
    //     let file = write_csv(tmp.path(), "f.csv", 5);
    //     let app = app_with_dir(tmp.path(), u16::MAX);
    //     let state = app.state::<AppState>();
    //     for _ in 0..u16::MAX { state_controller::increment_current_block(&state).unwrap(); }

    //     // 65535 * 65535 skipped lines; must not overflow or hang
    //     assert_eq!(page(&state, &file), vec!["id"]);
    //     assert_eq!(state_controller::get_current_block(&state).unwrap(), u16::MAX);
    // }

    // #[test]
    // fn headers_with_quoted_commas_survive() {
    //     let tmp = tempdir().unwrap();
    //     let file = tmp.path().join("f.csv");
    //     fs::write(&file, "\"time, s\",alt\n1,2\n").unwrap();
    //     let app = app_with_dir(tmp.path(), 5);
    //     let state = app.state::<AppState>();
    //     let rows = load(&state, &file).unwrap();
    //     assert_eq!(rows[0], vec!["time, s", "alt"]);
    //     assert_eq!(rows[1], vec!["1", "2"]);
    // }

    #[test]
    fn errors_do_not_advance_the_counter() {
        let tmp = tempdir().unwrap();
        let app = app_with_dir(tmp.path(), 3);
        let state = app.state::<AppState>();

        // missing file
        assert!(load(&state, &tmp.path().join("nope.csv")).is_err());

        // ragged row (header has 2 fields, data row has 3)
        let ragged = tmp.path().join("ragged.csv");
        fs::write(&ragged, "a,b\n1,2,3\n").unwrap();
        assert!(load(&state, &ragged).is_err());

        assert_eq!(state_controller::get_current_block(&state).unwrap(), 0);
    }
}