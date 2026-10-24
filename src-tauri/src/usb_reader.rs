use serde::Serialize;
use serialport::{SerialPortType, UsbPortInfo};
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
    let mut seen = 0;

    for part in line.split('|') {
        let (key, val) = part.split_once(':')?;
        let val: f32 = val.trim().parse().ok()?;
        match key.trim() {
            "Acceleration X" => t.accel_x = val,
            "Acceleration Y" => t.accel_y = val,
            "Acceleration Z" => t.accel_z = val,
            "Rotation X" => t.rot_x = val,
            "Rotation Y" => t.rot_y = val,
            "Rotation Z" => t.rot_z = val,
            "Temperature" => t.temperature = val,
            _ => continue,
        }
        seen += 1;
    }
    // Only accept lines containing all 7 fields (rejects boot noise / partial lines)
    (seen == 7).then_some(t)
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