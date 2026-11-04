use serde::Serialize;

use crate::telem_decoder::{LinkInfo, PAYLOAD_LEN};

/// Latest known value of each packet type (like AltOS' AltosState).
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlightState {
    pub imu: Option<MegaImu>,
    pub kalman: Option<MegaKalman>,
    pub gps: Option<Location>,
    pub sats: Option<Satellites>,
    pub config: Option<Config>,
    pub sensor: Option<Sensor>,
    pub comp: Option<Companion>,
    pub mini: Option<MiniSensor>,
    pub metrum_sens: Option<MetrumSensor>,
    pub calib: Option<MetrumCalibration>,
}

/// One merged row of flight data, emitted per IMU/Kalman packet.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub uptime_ms: u64, // wrap-corrected device clock
    pub time_s: f64,    // relative to the first "boost" report
    pub link: LinkInfo,
    pub latest: FlightState,
}

/// What happened while reading a file, so the UI can show "N corrupt packets skipped".
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadStats {
    pub lines: usize,
    pub ok: usize,
    pub crc_failed: usize,
    pub other_errors: usize,
}

#[derive(Serialize, Debug, Clone)]
#[serde(tag="type")]
pub enum Payload {
    Sensor(Sensor),         // 0x01 / 0x02 / 0x03 (TeleMetrum v1, TeleMini v1, TeleNano)
    Config(Config),         // 0x04
    Location(Location),     // 0x05
    Satellites(Satellites), // 0x06
    Companion(Companion),   // 0x07
    MegaImu(MegaImu),       // 0x08 / 0x12 / 0x13 / 0x14 (same layout, different sensors)
    MegaKalman(MegaKalman), // 0x09 / 0x15 (differ in voltage scaling)
    MiniSensor(MiniSensor), // 0x11 (TeleMini v3)
    MetrumSensor(MetrumSensor), // 0x0A (TeleMetrum v2 Sensor)
    MetrumCalibration(MetrumCalibration), // 0x0B (TeleMetrum v2 Calibration)

    Unknown { type_id: u8, raw: [u8; PAYLOAD_LEN] },
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all="camelCase")]
pub struct Sensor {
    pub state: u8,          // Flight State
    pub accel: i16,         // Accelerometer (Not TeleNano/TeleMini)
    pub pres: i16,          // Pressure Sensor
    pub temp: i16,          // Temperature Sensor
    pub v_batt: i16,        // Battery Voltage
    pub sense_d: i16,       // Drogue Continuity Sense (Not TeleNano)
    pub sense_m: i16,       // Main Continuity (Not TeleNano)
    pub acceleration: i16,  // m/s^2 * 16
    pub speed: i16,         // m/s * 16
    pub height: i16,        // m
    pub ground_pres: i16,   // Average Barometer Reading on Ground
    pub ground_accel: i16,  // (Not TeleNano/TeleMini)
    pub accel_plus_g: i16,  // (Not TeleNano/TeleMini)
    pub accel_minus_g: i16, // (Not TeleNano/TeleMini)
    pub kind: SensorKind,
}

#[derive(Serialize, Debug, Clone)]
pub enum SensorKind {
    TeleMetrumV1,
    TeleMiniV1,
    TeleNano,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all="camelCase")]
pub struct Config {
    pub dev_type: u8,        // Device Type
    pub flight: u16,         // Flight Number
    pub config_major: u8,    // Config Major Version
    pub config_minor: u8,    // Config Minor Version
    pub apogee_delay: u16,   // Apogee Deploy Delay (seconds)
    pub main_deploy: u16,    // Main Deploy Alt (meters)
    pub flight_log_max: u16, // Maximum Flight Log Size (kB)
    pub callsign: [u8; 8],   // Radio Operator Identifier (chars)
    pub version: [u8; 8],    // Software Version Identifier (chars)
                         // C++'s characters are 1 byte while Rust's are 4
                         // Using u8 since it's 1 byte to get around this
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all="camelCase")]
pub struct Location {
    pub flags: GpsFlags, // Based on the first byte of information
    pub altitude: i16,   // m
    pub latitude: i32,   // degrees * 10^7
    pub longitude: i32,  // degrees * 10^7
    pub year: u8,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub pdop: u8,      // * 5
    pub hdop: u8,      // * 5
    pub vdop: u8,      // * 5
    pub mode: GpsMode, // 1 byte
    pub ground_speed: u16,
    pub climb_rate: i16,
    pub course: u8,
    // 1 Byte of padding
}

#[derive(Serialize, Debug, Clone)]
pub enum GpsMode {
    NotValid,     // All Data is Invalid
    Autonomous,   // Data is Derived from Satellite Data
    Differential, // Data is Augmented with Differential Data from a Known Ground Station
    Estimated,    // Data is Estimated using Dead Reckoning from the Last Known Data
    Manual,       // Data was Entered Manually
    Simulated,    // GPS Receiver Testing Mode
    Other(u8),
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all="camelCase")]
pub struct GpsFlags {
    pub nsats: u8,          // Bits 0-3, Number of Satellites in Solution
    pub valid: bool,        // Bit 4, GPS Solution is Valid
    pub running: bool,      // Bit 5, GPS Receiver is Operational
    pub date_valid: bool,   // Bit 6, Reported Date is valid
    pub course_valid: bool, // Bit 7, Ground Speed, Course, & Climb Rates are Valid
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all="camelCase")]
pub struct Satellites {
    pub channels: u8, // Number of Reported Satellite Information
    pub sats: [SatelliteInfo; 12], // Up to 12 channels of Satellite Information (2 bytes each)
    // 2 Bytes of Padding
}

#[derive(Serialize, Debug, Clone, Copy, Default)]
#[serde(rename_all="camelCase")]
pub struct SatelliteInfo {
    pub svid: u8,  // Space Vehicle Identifier
    pub c_n_1: u8, // C/N1 signal quality indicator
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all="camelCase")]
pub struct Companion {
    pub board_id: u8,              // Type of Companion Board Attached
    pub update_period: u8,         // How often Telemetry is sent (1/100ths of a second)
    pub channels: u8,              // Number of Data Channels Supplied
    pub companion_data: [u16; 12], // Up to 12 channels of 16-bit Companion Data
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all="camelCase")]
pub struct MegaImu {
    pub orient: u8,   // Angle from Vertical (degrees)
    pub accel: i16,   // High G Accelerometer
    pub pres: i32,    // Pressure (Pa)
    pub temp: i16,    // Temperature (°C * 100)
    pub accel_x: i16, // X Axis Acceleration (across)
    pub accel_y: i16, // Y Axis Acceleration (Along)
    pub accel_z: i16, // Z Axis Acceleration (Through)
    pub gyro_x: i16,  // X Axis Rotation (Roll)
    pub gyro_y: i16,  // Y Axis Rotation (Pitch)
    pub gyro_z: i16,  // Z Axis Rotation (Yaw)
    pub mag_x: i16,   // X Field Strength (Across)
    pub mag_y: i16,   // Y Field Strength (Along)
    pub mag_z: i16,   // Z Field Strength (Through)
    pub kind: ImuKind,
}

#[derive(Serialize, Debug, Clone)]
pub enum ImuKind {
    Invensense,
    BMX160,
    MPU6000,
    BMI088,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all="camelCase")]
pub struct MegaKalman {
    pub state: u8,          // Flight State
    pub v_batt: i16,        // Battery Voltage
    pub v_pyro: i16,        // Pyro Battery Voltage
    pub sense: [u8; 6],     // Pyro Continuity Sense
    pub ground_pres: i32,   // Average Barometer Reading on Ground
    pub ground_accel: i16,  // Average Accelerometer Reading on Ground
    pub accel_plus_g: i16,  // Accel Calibration at +1g
    pub accel_minus_g: i16, // Accel Calibration at -1g
    pub acceleration: i16,  // m/s^2 * 16
    pub speed: i16,         // m/s * 16
    pub height: i16,        // m
    pub voltage: VoltageRange,
}

#[derive(Serialize, Debug, Clone)]
pub enum VoltageRange {
    V15,
    V30,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all="camelCase")]
pub struct MiniSensor {
    pub state: u8,         // Flight State
    pub v_batt: i16,       // Battery Voltage
    pub sense_a: i16,      // Apogee Continuity Sense
    pub sense_m: i16,      // Main Continuity Sense
    pub pres: i32,         // Pressure Sensor (Pa * 10)
    pub temp: i16,         // Temperature Sensor (°C * 100)
    pub acceleration: i16, // m/s^2 * 16
    pub speed: i16,        // m/s * 16
    pub height: i16,       // m
    pub ground_pres: i32,  // Average Barometer Reading on Ground
                       // 4 bytes of padding
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all="camelCase")]
pub struct MetrumSensor {
    pub state: u8,         // Flight State
    pub accel: i16,        // Accelerometer
    pub pres: i32,         // Pressure Sensor (Pa * 10)
    pub temp: i16,         // Temperature Sensor (°C * 100)
    pub acceleration: i16, // m/s^2 * 16
    pub speed: i16,        // m/s * 16
    pub height: i16,       // m
    pub v_batt: i16,       // Battery Voltage
    pub sense_d: i16,      // Drogue Continuity Sense
    pub sense_m: i16,      // Main Continuity Sense
                       // 6 Bytes of padding
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all="camelCase")]
pub struct MetrumCalibration {
    // 3 bytes of padding
    pub ground_pres: i32,  // Average Barometer Reading on Ground
    pub ground_accel: i16, // Average Accelerometer Reading on Ground
    pub accel_plus_g: i16, // Accel calibration at +1g
    pub accel_minus_g: i16, // Accel calibration at -1g
                       // 14 bytes of padding
}