use crate::{state_controller, telem_decoder};
use crate::telem_models::{LoadStats, Snapshot};

use csv::StringRecord;
use std::{fs, io::BufReader, io::BufRead, path::Path };
use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum FlightData {
    Table(Vec<Vec<String>>),
    Telemetry { snapshots: Vec<Snapshot>, stats: LoadStats },
}

impl FlightData {
    fn len(&self) -> usize {
        match self {
            FlightData::Table(vec) => vec.len(),
            FlightData::Telemetry { snapshots: packets, .. } => packets.len() 
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

fn read_telem_snapshots(path: &str) -> Result<(Vec<Snapshot>, LoadStats), String> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;

    // Stop at the first unreadable line instead of looping on a persistent I/O error.
    let mut read_error: Option<String> = None;
    let lines = BufReader::new(file).lines().map_while(|l| match l {
        Ok(line) => Some(line),
        Err(e) => {
            read_error = Some(e.to_string());
            None
        }
    });

    let result = telem_decoder::build_snapshots(lines);
    match read_error {
        Some(e) => Err(format!("Failed reading telemetry file: {e}")),
        None => Ok(result),
    }
}

async fn load_all_telem_data(path: String) -> Result<FlightData, String> {
    let (snapshots, stats) = read_telem_snapshots(&path)?;
    Ok(FlightData::Telemetry { snapshots, stats })
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
    let block = state_controller::get_current_block(state)? as usize;
    let block_size = state_controller::get_block_size(state.clone())? as usize;

    // Paging counts valid snapshots, not raw lines, so corrupt lines never eat into a block.
    let (snapshots, stats) = read_telem_snapshots(&path)?;
    let page: Vec<Snapshot> = snapshots
        .into_iter()
        .skip(block * block_size)
        .take(block_size)
        .collect();

    Ok(FlightData::Telemetry { snapshots: page, stats })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_controller::AppConfig;
    use crate::state_controller::AppState;
    use crate::telem_decoder::{PACKET_LEN, PAYLOAD_LEN, PAD_LEAD_S};
    use std::future::Future;
    use std::path::{Path, PathBuf};
    use tauri::test::{mock_app, MockRuntime};
    use tauri::Manager;
    use tempfile::tempdir;

    // ───────────────────────── generic helpers ─────────────────────────

    fn run<F: Future>(f: F) -> F::Output { tauri::async_runtime::block_on(f) }

    fn app_with_dir(dir: &Path, block_size: u16) -> tauri::App<MockRuntime> {
        let app = mock_app();
        app.manage(AppState::from(AppConfig {
            data_path: dir.to_path_buf(),
            loaded_block_size: block_size,
        }));
        app
    }

    fn load_all(file: &Path) -> Result<FlightData, String> {
        run(load_all_flight_data(file.to_str().unwrap().to_string()))
    }

    fn load_block(state: &tauri::State<'_, AppState>, file: &Path) -> Result<FlightData, String> {
        run(load_flight_block(state.clone(), file.to_str().unwrap().to_string()))
    }

    fn counter(state: &tauri::State<'_, AppState>) -> u16 {
        state_controller::get_current_block(state).unwrap()
    }

    fn rows_of(d: FlightData) -> Vec<Vec<String>> {
        match d { FlightData::Table(r) => r, _ => panic!("expected FlightData::Table") }
    }

    fn telem_of(d: FlightData) -> (Vec<Snapshot>, LoadStats) {
        match d {
            FlightData::Telemetry { snapshots, stats } => (snapshots, stats),
            _ => panic!("expected FlightData::Telemetry"),
        }
    }

    fn err_of(r: Result<FlightData, String>) -> String {
        match r { Err(e) => e, Ok(_) => panic!("expected an error") }
    }

    /// first column of each CSV row of a block, e.g. ["id", "r0", "r1"]
    fn page(state: &tauri::State<'_, AppState>, file: &Path) -> Vec<String> {
        rows_of(load_block(state, file).unwrap()).into_iter().map(|r| r[0].clone()).collect()
    }

    fn names(paths: Vec<String>) -> Vec<String> {
        let mut v: Vec<String> = paths.iter()
            .map(|p| Path::new(p).file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        v.sort();
        v
    }

    // "id,val" header + rows r0,v0 / r1,v1 / ...
    fn write_csv(dir: &Path, name: &str, rows: usize) -> PathBuf {
        let mut s = String::from("id,val\n");
        for i in 0..rows { s.push_str(&format!("r{i},v{i}\n")); }
        let p = dir.join(name);
        fs::write(&p, s).unwrap();
        p
    }

    fn write_lines(dir: &Path, name: &str, lines: &[String], eol: &str) -> PathBuf {
        let p = dir.join(name);
        fs::write(&p, format!("{}{}", lines.join(eol), eol)).unwrap();
        p
    }

    // ───────────────────────── telemetry builders ─────────────────────────
    // Frames are built in code (checksum computed), so no hand-copied hex.

    const GOOD_LQI: u8 = 0x88; // bit 7 set = CRC OK
    const BAD_LQI: u8 = 0x14;  // bit 7 clear = corrupt packet

    fn telem_bytes(type_id: u8, tick: u16, payload: &[u8; PAYLOAD_LEN], lqi: u8) -> Vec<u8> {
        let mut b = vec![(PACKET_LEN + 2) as u8];
        b.extend_from_slice(&0x3ffe_u16.to_le_bytes()); // serial
        b.extend_from_slice(&tick.to_le_bytes());
        b.push(type_id);
        b.extend_from_slice(payload);
        b.push((-90i8) as u8); // rssi
        b.push(lqi);
        let sum = b[1..].iter().fold(0x5a_u8, |a, &x| a.wrapping_add(x));
        b.push(sum);
        b
    }

    fn to_line(bytes: &[u8]) -> String { format!("TELEM {}", hex::encode(bytes)) }

    fn telem_line(type_id: u8, tick: u16, payload: &[u8; PAYLOAD_LEN], lqi: u8) -> String {
        to_line(&telem_bytes(type_id, tick, payload, lqi))
    }

    fn kalman_payload(state: u8) -> [u8; PAYLOAD_LEN] {
        let mut p = [0u8; PAYLOAD_LEN];
        p[0] = state;
        p
    }

    fn imu_payload(pres: i32) -> [u8; PAYLOAD_LEN] {
        let mut p = [0u8; PAYLOAD_LEN];
        p[0] = 10;                                     // orient
        p[3..7].copy_from_slice(&pres.to_le_bytes());  // after orient (1) + accel (2)
        p
    }

    fn corrupt_line() -> String { telem_line(0x14, 5, &imu_payload(1), BAD_LQI) }

    /// 4 pad packets, then boost at tick 1203 (t = 0). With PAD_LEAD_S = 2.0 only the first packet
    /// (t = -2.03 s) is cut: 7 snapshots at -2.00, -1.03, -1.00, -0.03, 0.00, 0.97, 1.00 s.
    fn flight_lines() -> Vec<String> {
        vec![
            telem_line(0x14, 1000, &imu_payload(90_822), GOOD_LQI), // pad
            telem_line(0x15, 1003, &kalman_payload(2), GOOD_LQI),   // pad
            telem_line(0x14, 1100, &imu_payload(90_822), GOOD_LQI), // pad
            telem_line(0x15, 1103, &kalman_payload(2), GOOD_LQI),   // pad
            telem_line(0x14, 1200, &imu_payload(90_800), GOOD_LQI), // before the boost report: dropped
            telem_line(0x15, 1203, &kalman_payload(3), GOOD_LQI),   // first boost report: t = 0
            telem_line(0x14, 1300, &imu_payload(90_000), GOOD_LQI),
            telem_line(0x15, 1303, &kalman_payload(4), GOOD_LQI),
        ]
    }

    fn assert_times(snaps: &[Snapshot], expected: &[f64]) {
        let got: Vec<f64> = snaps.iter().map(|s| s.time_s).collect();
        assert_eq!(got.len(), expected.len(), "got {got:?}");
        for (g, e) in got.iter().zip(expected) {
            assert!((g - e).abs() < 1e-9, "got {got:?}, expected {expected:?}");
        }
    }

    // ───────────────────────── get_all_saved_flights ─────────────────────────

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
        assert_eq!(counter(&state), 0);
    }

    // ───────────────────────── FlightData / dispatch ─────────────────────────

    #[test]
    fn flight_data_len_counts_the_header_row_for_tables() {
        // load_flight_block relies on this: an "empty" table page has len 1, an empty telemetry page 0
        assert_eq!(FlightData::Table(vec![vec!["id".to_string()]]).len(), 1);
        let empty = FlightData::Telemetry { snapshots: vec![], stats: LoadStats::default() };
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn unsupported_extensions_are_rejected_by_both_commands() {
        let tmp = tempdir().unwrap();
        let app = app_with_dir(tmp.path(), 3);
        let state = app.state::<AppState>();

        // eeprom is listed by get_all_saved_flights but cannot be loaded yet
        for name in ["a.txt", "b.eeprom", "noext", "c.csv.bak"] {
            let f = tmp.path().join(name);
            fs::write(&f, "x").unwrap();
            assert_eq!(err_of(load_all(&f)), "Invalid file extension", "{name}");
            assert_eq!(err_of(load_block(&state, &f)), "Invalid file extension", "{name}");
        }
        assert_eq!(counter(&state), 0);
    }

    #[test]
    fn extension_dispatch_is_case_insensitive_for_loading() {
        let tmp = tempdir().unwrap();
        let csv = write_csv(tmp.path(), "UPPER.CSV", 2);
        assert_eq!(rows_of(load_all(&csv).unwrap()).len(), 3);

        let telem = write_lines(tmp.path(), "FLIGHT.TELEM", &flight_lines(), "\n");
        assert_eq!(telem_of(load_all(&telem).unwrap()).0.len(), 7);
    }

    // ───────────────────────── CSV: load_all_flight_data ─────────────────────────

    #[test]
    fn csv_load_all_returns_every_row_including_the_header() {
        let tmp = tempdir().unwrap();
        let file = write_csv(tmp.path(), "f.csv", 3);
        let rows = rows_of(load_all(&file).unwrap());
        assert_eq!(rows.len(), 4);
        assert_eq!(rows[0], vec!["id", "val"]);
        assert_eq!(rows[3], vec!["r2", "v2"]);
    }

    #[test]
    fn csv_load_all_handles_quotes_embedded_newlines_crlf_and_unicode() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("f.csv");
        fs::write(&file, "\"a,b\",\"x\ny\",z\r\n日本,\"say \"\"hi\"\"\",é\r\n").unwrap();
        let rows = rows_of(load_all(&file).unwrap());
        assert_eq!(rows[0], vec!["a,b", "x\ny", "z"]);
        assert_eq!(rows[1], vec!["日本", "say \"hi\"", "é"]);
    }

    #[test]
    fn csv_load_all_empty_file_gives_no_rows() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("empty.csv");
        fs::write(&file, "").unwrap();
        assert!(rows_of(load_all(&file).unwrap()).is_empty());
    }

    #[test]
    fn csv_load_all_missing_file_is_an_error() {
        let tmp = tempdir().unwrap();
        assert!(load_all(&tmp.path().join("nope.csv")).is_err());
    }

    #[test]
    fn csv_load_all_ragged_rows_are_an_error_not_a_panic() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("ragged.csv");
        fs::write(&file, "a,b\n1,2,3\n").unwrap();
        assert!(load_all(&file).is_err());
    }

    // ───────────────────────── CSV: load_flight_block ─────────────────────────

    #[test]
    fn csv_pages_through_a_file_block_by_block() {
        let tmp = tempdir().unwrap();
        let file = write_csv(tmp.path(), "f.csv", 7);
        let app = app_with_dir(tmp.path(), 3);
        let state = app.state::<AppState>();

        assert_eq!(page(&state, &file), vec!["id", "r0", "r1", "r2"]);
        assert_eq!(page(&state, &file), vec!["id", "r3", "r4", "r5"]);
        assert_eq!(page(&state, &file), vec!["id", "r6"]); // partial last block
        assert_eq!(counter(&state), 3);

        // Past the end: headers only, and the counter stops advancing
        assert_eq!(page(&state, &file), vec!["id"]);
        assert_eq!(page(&state, &file), vec!["id"]);
        assert_eq!(counter(&state), 3);
    }

    #[test]
    fn csv_exact_multiple_of_block_size_has_no_trailing_empty_block() {
        let tmp = tempdir().unwrap();
        let file = write_csv(tmp.path(), "f.csv", 6);
        let app = app_with_dir(tmp.path(), 3);
        let state = app.state::<AppState>();

        page(&state, &file);
        assert_eq!(counter(&state), 1);
        page(&state, &file);
        assert_eq!(counter(&state), 2);
        assert_eq!(page(&state, &file), vec!["id"]); // headers only...
        assert_eq!(counter(&state), 2);              // ...and the counter did not advance
    }

    #[test]
    fn csv_reset_restarts_paging_from_the_first_block() {
        let tmp = tempdir().unwrap();
        let file = write_csv(tmp.path(), "f.csv", 7);
        let app = app_with_dir(tmp.path(), 3);
        let state = app.state::<AppState>();

        page(&state, &file);
        page(&state, &file);
        state_controller::reset_current_block(&state).unwrap();
        assert_eq!(page(&state, &file), vec!["id", "r0", "r1", "r2"]);
    }

    #[test]
    fn csv_block_size_larger_than_file_returns_everything_once() {
        let tmp = tempdir().unwrap();
        let file = write_csv(tmp.path(), "f.csv", 4);
        let app = app_with_dir(tmp.path(), 1000);
        let state = app.state::<AppState>();

        assert_eq!(page(&state, &file), vec!["id", "r0", "r1", "r2", "r3"]);
        assert_eq!(page(&state, &file), vec!["id"]);
    }

    #[test]
    fn csv_header_only_file_returns_just_the_header_and_never_advances() {
        let tmp = tempdir().unwrap();
        let file = write_csv(tmp.path(), "f.csv", 0);
        let app = app_with_dir(tmp.path(), 3);
        let state = app.state::<AppState>();
        assert_eq!(page(&state, &file), vec!["id"]);
        assert_eq!(counter(&state), 0);
    }

    #[test]
    fn csv_block_size_zero_returns_only_headers_and_never_advances() {
        let tmp = tempdir().unwrap();
        let file = write_csv(tmp.path(), "f.csv", 5);
        let app = app_with_dir(tmp.path(), 0);
        let state = app.state::<AppState>();
        for _ in 0..3 {
            assert_eq!(page(&state, &file), vec!["id"]);
        }
        assert_eq!(counter(&state), 0); // the old "counter still advances" quirk is gone
    }

    #[test]
    fn csv_index_times_size_uses_u32_and_cannot_overflow() {
        let tmp = tempdir().unwrap();
        let file = write_csv(tmp.path(), "f.csv", 5);
        let app = app_with_dir(tmp.path(), u16::MAX);
        let state = app.state::<AppState>();
        for _ in 0..u16::MAX { state_controller::increment_current_block(&state).unwrap(); }

        // 65535 * 65535 skipped lines: must neither overflow nor hang
        assert_eq!(page(&state, &file), vec!["id"]);
        assert_eq!(counter(&state), u16::MAX);
    }

    #[test]
    fn csv_headers_with_quoted_commas_survive() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("f.csv");
        fs::write(&file, "\"time, s\",alt\n1,2\n").unwrap();
        let app = app_with_dir(tmp.path(), 5);
        let state = app.state::<AppState>();
        let rows = rows_of(load_block(&state, &file).unwrap());
        assert_eq!(rows[0], vec!["time, s", "alt"]);
        assert_eq!(rows[1], vec!["1", "2"]);
    }

    #[test]
    fn errors_do_not_advance_the_counter() {
        let tmp = tempdir().unwrap();
        let app = app_with_dir(tmp.path(), 3);
        let state = app.state::<AppState>();

        // missing files (both formats)
        assert!(load_block(&state, &tmp.path().join("nope.csv")).is_err());
        assert!(load_block(&state, &tmp.path().join("nope.telem")).is_err());

        // ragged CSV row (header has 2 fields, data row has 3)
        let ragged = tmp.path().join("ragged.csv");
        fs::write(&ragged, "a,b\n1,2,3\n").unwrap();
        assert!(load_block(&state, &ragged).is_err());

        // unreadable telemetry (invalid UTF-8)
        let bad = tmp.path().join("bad.telem");
        fs::write(&bad, b"\xff\xfe\n").unwrap();
        assert!(load_block(&state, &bad).is_err());

        assert_eq!(counter(&state), 0);
    }

    // ───────────────────────── TELEM: load_all_flight_data ─────────────────────────

    #[test]
    fn telem_load_all_keeps_the_pad_lead_and_merges_packets() {
        let tmp = tempdir().unwrap();
        let file = write_lines(tmp.path(), "f.telem", &flight_lines(), "\n");
        let (snaps, stats) = telem_of(load_all(&file).unwrap());

        assert_times(&snaps, &[-2.0, -1.03, -1.0, -0.03, 0.0, 0.97, 1.0]);
        assert_eq!(snaps[0].uptime_ms, 10_030);
        assert_eq!(snaps[4].uptime_ms, 12_030); // the boost report is t = 0
        assert!(snaps.iter().all(|s| s.time_s >= -PAD_LEAD_S));

        let state = |i: usize| snaps[i].latest.kalman.as_ref().unwrap().state;
        let pres = |i: usize| snaps[i].latest.imu.as_ref().unwrap().pres;

        // Merging happens BEFORE the lead filter: the first kept row (a Kalman packet)
        // still carries the pressure from the IMU packet that was itself cut.
        assert_eq!((state(0), pres(0)), (2, 90_822));
        assert_eq!((state(3), pres(3)), (2, 90_800)); // pad IMU just before the boost report
        assert_eq!((state(4), pres(4)), (3, 90_800)); // boost report + that IMU
        assert_eq!((state(5), pres(5)), (3, 90_000)); // new IMU, Kalman carried over
        assert_eq!((state(6), pres(6)), (4, 90_000)); // new Kalman, IMU carried over

        assert_eq!((stats.lines, stats.ok, stats.crc_failed, stats.other_errors), (8, 8, 0, 0));
    }

    #[test]
    fn telem_without_a_boost_report_starts_at_the_first_packet() {
        let tmp = tempdir().unwrap();
        let pad_only: Vec<String> = flight_lines().into_iter().take(4).collect();
        let file = write_lines(tmp.path(), "pad.telem", &pad_only, "\n");
        let (snaps, _) = telem_of(load_all(&file).unwrap());

        assert_times(&snaps, &[0.0, 0.03, 1.0, 1.03]); // nothing dropped, t = 0 at the first tick
    }

    #[test]
    fn telem_clock_wrap_across_the_file_gives_continuous_time() {
        let tmp = tempdir().unwrap();
        let lines = vec![
            telem_line(0x15, 65_500, &kalman_payload(3), GOOD_LQI), // boost, near the end of the counter
            telem_line(0x14, 65_520, &imu_payload(90_000), GOOD_LQI),
            telem_line(0x14, 20, &imu_payload(89_000), GOOD_LQI),   // counter wrapped
        ];
        let file = write_lines(tmp.path(), "wrap.telem", &lines, "\n");
        let (snaps, _) = telem_of(load_all(&file).unwrap());

        assert_times(&snaps, &[0.0, 0.2, 0.56]);
    }

    #[test]
    fn telem_stats_count_every_kind_of_bad_line() {
        let tmp = tempdir().unwrap();
        let mut bad_checksum = telem_bytes(0x14, 7, &imu_payload(1), GOOD_LQI);
        *bad_checksum.last_mut().unwrap() ^= 0xff;

        let mut lines = flight_lines(); // 8 good
        lines.push(corrupt_line());
        lines.push(corrupt_line());
        lines.push("TELEM zz".to_string());       // bad hex
        lines.push("not telemetry".to_string());  // bad prefix
        lines.push(String::new());                // blank lines are not counted
        lines.push(to_line(&bad_checksum));       // checksum mismatch

        let file = write_lines(tmp.path(), "mixed.telem", &lines, "\n");
        let (_, stats) = telem_of(load_all(&file).unwrap());

        assert_eq!(stats.lines, 13);
        assert_eq!(stats.ok, 8);
        assert_eq!(stats.crc_failed, 2);
        assert_eq!(stats.other_errors, 3);
    }

    #[test]
    fn telem_crlf_and_blank_lines_are_handled() {
        let tmp = tempdir().unwrap();
        let mut lines = flight_lines();
        lines.insert(2, String::new());
        let file = write_lines(tmp.path(), "crlf.telem", &lines, "\r\n");
        let (snaps, stats) = telem_of(load_all(&file).unwrap());

        assert_eq!(snaps.len(), 7);
        assert_eq!((stats.lines, stats.ok), (8, 8));
    }

    #[test]
    fn telem_file_without_flight_data_gives_no_snapshots() {
        let tmp = tempdir().unwrap();
        let lines = vec![
            telem_line(0x04, 10, &[0u8; PAYLOAD_LEN], GOOD_LQI), // config
            telem_line(0x05, 11, &[0u8; PAYLOAD_LEN], GOOD_LQI), // GPS
            telem_line(0x7f, 12, &[0u8; PAYLOAD_LEN], GOOD_LQI), // unknown type: decodes, but no snapshot
        ];
        let file = write_lines(tmp.path(), "noflight.telem", &lines, "\n");
        let (snaps, stats) = telem_of(load_all(&file).unwrap());

        assert!(snaps.is_empty());
        assert_eq!((stats.lines, stats.ok), (3, 3));
    }

    #[test]
    fn telem_empty_file_is_not_an_error() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("empty.telem");
        fs::write(&file, "").unwrap();
        let (snaps, stats) = telem_of(load_all(&file).unwrap());
        assert!(snaps.is_empty());
        assert_eq!((stats.lines, stats.ok, stats.crc_failed, stats.other_errors), (0, 0, 0, 0));
    }

    #[test]
    fn telem_missing_file_and_invalid_utf8_are_errors() {
        let tmp = tempdir().unwrap();
        assert!(load_all(&tmp.path().join("nope.telem")).is_err());

        let bad = tmp.path().join("bad.telem");
        fs::write(&bad, b"TELEM abc\n\xff\xfe\n").unwrap();
        assert!(err_of(load_all(&bad)).contains("Failed reading telemetry file"));
    }

    // ───────────────────────── TELEM: load_flight_block ─────────────────────────

    #[test]
    fn telem_pages_through_snapshots_block_by_block() {
        let tmp = tempdir().unwrap();
        let file = write_lines(tmp.path(), "f.telem", &flight_lines(), "\n");
        let app = app_with_dir(tmp.path(), 2);
        let state = app.state::<AppState>();

        let (p0, _) = telem_of(load_block(&state, &file).unwrap());
        assert_times(&p0, &[-2.0, -1.03]);
        assert_eq!(counter(&state), 1);

        let (p1, _) = telem_of(load_block(&state, &file).unwrap());
        assert_times(&p1, &[-1.0, -0.03]);
        assert_eq!(counter(&state), 2);

        let (p2, _) = telem_of(load_block(&state, &file).unwrap());
        assert_times(&p2, &[0.0, 0.97]);
        assert_eq!(counter(&state), 3);

        let (p3, _) = telem_of(load_block(&state, &file).unwrap());
        assert_times(&p3, &[1.0]); // partial last block
        assert_eq!(counter(&state), 4);

        let (p4, _) = telem_of(load_block(&state, &file).unwrap());
        assert!(p4.is_empty()); // past the end
        assert_eq!(counter(&state), 4); // an empty page does not advance the counter
    }

    #[test]
    fn telem_exact_multiple_of_block_size_has_no_trailing_empty_block() {
        let tmp = tempdir().unwrap();
        // 6 snapshots remain (-1.03 ... 1.00), all inside the lead window
        let lines: Vec<String> = flight_lines().into_iter().skip(2).collect();
        let file = write_lines(tmp.path(), "f.telem", &lines, "\n");
        let app = app_with_dir(tmp.path(), 3);
        let state = app.state::<AppState>();

        assert_eq!(telem_of(load_block(&state, &file).unwrap()).0.len(), 3);
        assert_eq!(counter(&state), 1);
        assert_eq!(telem_of(load_block(&state, &file).unwrap()).0.len(), 3);
        assert_eq!(counter(&state), 2);
        assert!(telem_of(load_block(&state, &file).unwrap()).0.is_empty());
        assert_eq!(counter(&state), 2); // no trailing empty block
    }

    #[test]
    fn telem_corrupt_lines_do_not_consume_block_space() {
        // Regression test for "586 of 600 lines skipped": blocks count valid snapshots, not raw lines
        let tmp = tempdir().unwrap();
        let mut lines = vec![corrupt_line(); 50];
        lines.extend(flight_lines());
        let file = write_lines(tmp.path(), "noisy.telem", &lines, "\n");
        let app = app_with_dir(tmp.path(), 3);
        let state = app.state::<AppState>();

        let (snaps, stats) = telem_of(load_block(&state, &file).unwrap());
        assert_eq!(snaps.len(), 3); // a full block despite 50 corrupt lines before the data
        assert_eq!((stats.lines, stats.ok, stats.crc_failed), (58, 8, 50)); // stats cover the whole file
    }

    #[test]
    fn telem_blocks_stitch_back_into_load_all() {
        let tmp = tempdir().unwrap();
        let file = write_lines(tmp.path(), "f.telem", &flight_lines(), "\n");
        let app = app_with_dir(tmp.path(), 2);
        let state = app.state::<AppState>();

        let all: Vec<u64> = telem_of(load_all(&file).unwrap()).0.iter().map(|s| s.uptime_ms).collect();

        let mut stitched = Vec::new();
        loop {
            let (page, _) = telem_of(load_block(&state, &file).unwrap());
            if page.is_empty() { break; }
            stitched.extend(page.iter().map(|s| s.uptime_ms));
        }
        assert_eq!(stitched, all);
    }

    #[test]
    fn telem_block_without_flight_data_never_advances() {
        let tmp = tempdir().unwrap();
        let lines = vec![telem_line(0x04, 10, &[0u8; PAYLOAD_LEN], GOOD_LQI)];
        let file = write_lines(tmp.path(), "cfg.telem", &lines, "\n");
        let app = app_with_dir(tmp.path(), 3);
        let state = app.state::<AppState>();

        assert!(telem_of(load_block(&state, &file).unwrap()).0.is_empty());
        assert_eq!(counter(&state), 0);
    }

    #[test]
    fn pad_lead_matches_the_fixtures() {
        assert_eq!(PAD_LEAD_S, 2.0, "these fixtures are written for a 2 s lead; update them if you change it");
    }

    #[test]
    fn telem_pad_lead_window_boundary_is_inclusive() {
        let tmp = tempdir().unwrap();
        let lines = vec![
            telem_line(0x14, 1000, &imu_payload(1), GOOD_LQI),    // t = -2.01: just outside, dropped
            telem_line(0x14, 1001, &imu_payload(2), GOOD_LQI),    // t = -2.00: on the edge, kept
            telem_line(0x14, 1002, &imu_payload(3), GOOD_LQI),    // t = -1.99: kept
            telem_line(0x15, 1201, &kalman_payload(3), GOOD_LQI), // boost report: t = 0
        ];
        let file = write_lines(tmp.path(), "edge.telem", &lines, "\n");
        let (snaps, _) = telem_of(load_all(&file).unwrap());

        assert_times(&snaps, &[-2.0, -1.99, 0.0]);
    }
}