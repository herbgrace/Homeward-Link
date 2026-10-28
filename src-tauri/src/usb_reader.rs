use serde::Serialize;
use serialport::{SerialPortType, UsbPortInfo};
use std::collections::HashSet;
use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};

// ---------- Data types ----------

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Telemetry {
    pub accel_x: f32,
    pub accel_y: f32,
    pub accel_z: f32,
    pub rot_x: f32,
    pub rot_y: f32,
    pub rot_z: f32,
    pub temperature: f32,
}

#[derive(Clone, Serialize)]
struct Status {
    connected: bool,
    device: Option<&'static str>,
    port: Option<String>,
}

/// "Acceleration X: 5.00 | Acceleration Y: 1.45 | ... | Temperature: 27.84"
fn parse_esp32(line: &str) -> Option<Telemetry> {
    let mut t = Telemetry::default();
    let mut seen: HashSet<&str> = HashSet::new();

    for part in line.split('|') {
        let (key, val) = part.split_once(':')?;
        let val: f32 = val.trim().parse().ok()?;

        let key = key.trim();

        if !seen.insert(key) {
            // Duplicate Key, don't overwrite data
            continue;
        }

        match key {
            "Acceleration X" => t.accel_x = val,
            "Acceleration Y" => t.accel_y = val,
            "Acceleration Z" => t.accel_z = val,
            "Rotation X" => t.rot_x = val,
            "Rotation Y" => t.rot_y = val,
            "Rotation Z" => t.rot_z = val,
            "Temperature" => t.temperature = val,
            _ => {
                seen.remove(key); // Not a valid key, shouldn't count towards seen
                continue;
            }
        }
    }
    // Only accept lines containing all 7 fields (rejects boot noise / partial lines)
    (seen.len() == 7).then_some(t)
}

// ---------- Device profiles ----------

struct DeviceProfile {
    name: &'static str,
    matches: fn(&UsbPortInfo) -> bool,
    baud: u32,
    init: Option<&'static [u8]>,
    parse: Option<fn(&str) -> Option<Telemetry>>,
}

fn product_contains(info: &UsbPortInfo, needle: &str) -> bool {
    info.product
        .as_deref()
        .map_or(false, |p| p.to_lowercase().contains(needle))
}

const PROFILES: &[DeviceProfile] = &[
    DeviceProfile {
        name: "ESP32",
        matches: |i| {
            matches!(
                (i.vid, i.pid),
                (0x10C4, 0xEA60) | (0x1A86, 0x7523) | (0x1A86, 0x55D4) | (0x0403, 0x6001)
            ) || i.vid == 0x303A
        },
        baud: 115_200,
        init: None,
        parse: Some(parse_esp32),
    },
    DeviceProfile {
        name: "TeleDongle",
        matches: |i| product_contains(i, "teledongle") || (i.vid == 0xFFFE && i.pid == 0x000C),
        baud: 115_200,
        init: None,  // fill in once you've confirmed the monitor command
        parse: None, // raw lines only until you write a TeleDongle parser
    },
];

fn find_device() -> Option<(String, &'static DeviceProfile)> {
    let ports = serialport::available_ports().ok()?;
    ports.into_iter().find_map(|p| {
        let SerialPortType::UsbPort(ref info) = p.port_type else {
            return None;
        };
        PROFILES
            .iter()
            .find(|prof| (prof.matches)(info))
            .map(|prof| (p.port_name.clone(), prof))
    })
}

// ---------- Worker thread ----------

const READ_TIMEOUT: Duration = Duration::from_millis(100);

/// Sleep up to `total`, waking early if `stop` is set.
fn interruptible_sleep(stop: &AtomicBool, total: Duration) {
    let step = Duration::from_millis(100);
    let mut waited = Duration::ZERO;
    while waited < total && !stop.load(Ordering::Relaxed) {
        thread::sleep(step);
        waited += step;
    }
}

fn run_session(
    app: &AppHandle,
    stop: &AtomicBool,
    port_name: &str,
    profile: &DeviceProfile,
) -> Result<(), Box<dyn Error>> {
    let mut port = serialport::new(port_name, profile.baud)
        .timeout(READ_TIMEOUT)
        .open()?;

    if let Some(cmd) = profile.init {
        port.write_all(cmd)?;
        port.flush()?;
    }

    let mut reader = BufReader::new(port);
    // NOTE: buf is only cleared after a complete line, so partial lines
    // survive read timeouts.
    let mut buf = Vec::new();

    while !stop.load(Ordering::Relaxed) {
        match reader.read_until(b'\n', &mut buf) {
            Ok(0) => return Err("port closed".into()),
            Ok(_) => {
                let line = String::from_utf8_lossy(&buf).trim_end().to_string();
                buf.clear();

                let _ = app.emit("serial-line", &line);
                if let Some(parse) = profile.parse {
                    if let Some(t) = parse(&line) {
                        let _ = app.emit("telemetry", &t);
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
            Err(e) => return Err(e.into()), // unplugged, etc.
        }
    }
    Ok(())
}

fn supervisor(app: AppHandle, stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::Relaxed) {
        match find_device() {
            Some((port_name, profile)) => {
                let _ = app.emit(
                    "serial-status",
                    Status { connected: true, device: Some(profile.name), port: Some(port_name.clone()) },
                );

                if let Err(e) = run_session(&app, &stop, &port_name, profile) {
                    eprintln!("Serial session ended: {e}");
                }

                let _ = app.emit(
                    "serial-status",
                    Status { connected: false, device: None, port: None },
                );
                interruptible_sleep(&stop, Duration::from_millis(500));
            }
            None => interruptible_sleep(&stop, Duration::from_secs(1)),
        }
    }
}

// ---------- Tauri state + commands ----------

struct Worker {
    stop: Arc<AtomicBool>,
    handle: JoinHandle<()>,
}

#[derive(Default)]
pub struct SerialManager(Mutex<Option<Worker>>);

impl SerialManager {
    pub fn stop(&self) {
        if let Some(w) = self.0.lock().unwrap().take() {
            w.stop.store(true, Ordering::Relaxed);
            let _ = w.handle.join();
        }
    }
}

impl Drop for SerialManager {
    fn drop(&mut self) {
        self.stop();
    }
}

#[tauri::command]
pub fn start_serial(app: AppHandle, state: State<SerialManager>) {
    let mut guard = state.0.lock().unwrap();
    if guard.is_some() {
        return; // already running
    }
    let stop = Arc::new(AtomicBool::new(false));
    let handle = {
        let stop = stop.clone();
        thread::spawn(move || supervisor(app, stop))
    };
    *guard = Some(Worker { stop, handle });
}

#[tauri::command]
pub fn stop_serial(state: State<SerialManager>) {
    state.stop();
}


#[cfg(test)]
mod tests {
    use super::*;

    const FULL: &str = "Acceleration X: 5.00 | Acceleration Y: 1.45 | Acceleration Z: -9.81 | \
                        Rotation X: 0.10 | Rotation Y: -0.20 | Rotation Z: 0.30 | Temperature: 27.84";

    fn usb(vid: u16, pid: u16, product: Option<&str>) -> UsbPortInfo {
        // If this fails to compile with "missing field `interface`", add `interface: None`
        UsbPortInfo { vid, pid, serial_number: None, manufacturer: None, product: product.map(String::from) }
    }
    fn profile(name: &str) -> &'static DeviceProfile {
        PROFILES.iter().find(|p| p.name == name).unwrap()
    }

    #[test]
    fn parses_a_complete_line() {
        let t = parse_esp32(FULL).unwrap();
        assert_eq!((t.accel_x, t.accel_y, t.accel_z), (5.0, 1.45, -9.81));
        assert_eq!((t.rot_x, t.rot_y, t.rot_z), (0.1, -0.2, 0.3));
        assert_eq!(t.temperature, 27.84);
    }

    #[test]
    fn rejects_incomplete_or_garbage_lines() {
        assert!(parse_esp32("").is_none());
        assert!(parse_esp32("ESP-ROM:esp32c3-api1-20210207").is_none());
        assert!(parse_esp32("Acceleration X: 5.00 | Acceleration Y: 1.45").is_none()); // partial
        assert!(parse_esp32(&FULL.replace("27.84", "warm")).is_none());                // non-numeric
        assert!(parse_esp32(&format!("{FULL} |")).is_none());                          // trailing separator
    }

    #[test]
    fn unknown_keys_are_ignored_but_do_not_count() {
        assert!(parse_esp32(&format!("{FULL} | Humidity: 40")).is_some());
        let without_temp = FULL.rsplit_once(" | ").unwrap().0;
        assert!(parse_esp32(&format!("{without_temp} | Humidity: 40")).is_none());
    }

    #[test]
    fn duplicate_key_does_not_stand_in_for_a_missing_one() {
        let without_temp = FULL.rsplit_once(" | ").unwrap().0;
        assert!(parse_esp32(&format!("{without_temp} | Acceleration X: 1.0")).is_none());
    }

    #[test]
    fn esp32_profile_matches_common_usb_serial_chips() {
        let p = profile("ESP32");
        for (vid, pid) in [(0x10C4, 0xEA60), (0x1A86, 0x7523), (0x1A86, 0x55D4), (0x0403, 0x6001), (0x303A, 0x1001)] {
            assert!((p.matches)(&usb(vid, pid, None)), "{vid:04x}:{pid:04x}");
        }
        assert!(!(p.matches)(&usb(0x1234, 0x5678, None)));
    }

    #[test]
    fn teledongle_profile_matches_by_ids_or_product_name() {
        let p = profile("TeleDongle");
        assert!((p.matches)(&usb(0xFFFE, 0x000C, None)));
        assert!((p.matches)(&usb(0x0000, 0x0000, Some("TeleDongle v3.0"))));
        assert!((p.matches)(&usb(0x0000, 0x0000, Some("my TELEDONGLE"))));
        assert!(!(p.matches)(&usb(0xFFFE, 0x000A, Some("TeleMetrum"))));
    }

    #[test]
    fn profiles_do_not_claim_each_others_devices() {
        // find_device() takes the first match, so overlap would cause misdetection
        let dongle = usb(0xFFFE, 0x000C, Some("TeleDongle"));
        assert!(!(profile("ESP32").matches)(&dongle));
        assert!(!(profile("TeleDongle").matches)(&usb(0x10C4, 0xEA60, None)));
    }
}