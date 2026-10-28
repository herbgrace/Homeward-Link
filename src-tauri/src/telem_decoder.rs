pub const PACKET_LEN: usize = 32; // Does not include length byte or checksum byte
pub const PAYLOAD_LEN: usize = PACKET_LEN - 5; // 27 data bytes

/// What the TeleDongle appended/prepended (not part of the radio packet)
#[derive(Debug, Clone, Copy)]
pub struct LinkInfo {
    pub rssi_dbm: f32,   // derived from raw i8: raw as f32 / 2.0 - 74.0
    pub lqi: u8,         // low 7 bits only
    pub crc_ok: bool,    // bit 7 of the raw LQI byte
}

/// Common to every packet
#[derive(Debug, Clone, Copy)]
pub struct Header {
    pub serial: u16, // Serial Number of Transmitting Device
    pub tick: u16, // Current Relative Timestamp (1/100ths of a second)
    pub type_id: u8, // What Type of Data is transmitted
}

#[derive(Debug, Clone)]
pub struct Telemetry {
    pub header: Header,
    pub link: LinkInfo,
    pub payload: Payload,
}

#[derive(Debug, Clone)]
pub enum Payload {
    Sensor(Sensor),                         // 0x01 / 0x02 / 0x03 (TeleMetrum v1, TeleMini v1, TeleNano)
    Config(Config),                         // 0x04
    Location(Location),                     // 0x05
    Satellites(Satellites),                 // 0x06
    Companion(Companion),                   // 0x07
    MegaImu(MegaImu),                       // 0x08 / 0x12 / 0x13 / 0x14 (same layout, different sensors)
    MegaKalman(MegaKalman),                 // 0x09 / 0x15 (differ in voltage scaling)
    MiniSensor(MiniSensor),                 // 0x11 (TeleMini v3)
    MetrumSensor(MetrumSensor),             // 0x0A (TeleMetrum v2 Sensor)
    MetrumCalibration(MetrumCalibration),   // 0x0B (TeleMetrum v2 Calibration)

    Unknown { type_id: u8, raw: [u8; PAYLOAD_LEN] },
}

pub trait PayloadDecode: Sized {
    fn decode(type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Self, DecodeError>;
}

#[derive(Debug, Clone)]
pub struct Location {
    flags: GpsFlags,
    altitude: i16, // m
    latitude: i32, // degrees * 10^7
    longitude: i32, // degrees * 10^7
    year: u8,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    pdop: u8, // * 5
    hdop: u8, // * 5
    vdop: u8, // * 5
    mode: GpsMode, // 1 byte
    ground_speed: u16,
    climb_rate: i16,
    course: u8,
    // 1 Byte of padding
}

impl PayloadDecode for Location {
    fn decode(type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Self, DecodeError> { todo!() }
}

#[derive(Debug, Clone)]
pub struct GpsFlags {
    nsats: u8, // Bits 0-3, Number of Satellites in Solution
    valid: bool, // Bit 4, GPS Solution is Valid
    running: bool, // Bit 5, GPS Receiver is Operational
    date_valid: bool, // Bit 6, Reported Date is valid
    course_valid: bool, // Bit 7, Ground Speed, Course, & Climb Rates are Valid
}

impl GpsFlags {
    fn from_byte(b: u8) -> Self {
        Self {
            nsats:        b & 0x0f,             // AND with 0000 1111 to get first 4
            valid:        b & (1 << 4) != 0,    // AND with 0001 0000
            running:      b & (1 << 5) != 0,    // AND with 0010 0000
            date_valid:   b & (1 << 6) != 0,    // AND with 0100 0000
            course_valid: b & (1 << 7) != 0,    // AND with 1000 0000
        }
    }
}

#[derive(Debug, Clone)]
pub enum GpsMode {
    NotValid, // All Data is Invalid
    Autonomous, // Data is Derived from Satellite Data
    Differential, // Data is Augmented with Differential Data from a Known Ground Station
    Estimated, // Data is Estimated using Dead Reckoning from the Last Known Data
    Manual, // Data was Entered Manually
    Simulated, // GPS Receiver Testing Mode
    Other(u8)
}

impl GpsMode {
    pub fn from_byte(b: u8) -> Self {
        match b {
            b'N' => Self::NotValid,
            b'A' => Self::Autonomous,
            b'D' => Self::Differential,
            b'E' => Self::Estimated,
            b'M' => Self::Manual,
            b'S' => Self::Simulated,
            other => Self::Other(other)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Satellites {
    channels: u8, // Number of Reported Satellite Information
    sats: [SatelliteInfo; 12], // Up to 12 channels of Satellite Information (2 bytes each)
    // 2 Bytes of Padding
}

impl PayloadDecode for Satellites {
    fn decode(type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Self, DecodeError> { todo!() }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SatelliteInfo {
    svid: u8, // Space Vehicle Identifier
    c_n_1: u8, // C/N1 signal quality indicator
}

#[derive(Debug, Clone)]
pub struct Sensor {
    state: u8, // Flight State
    accel: i16, // Accelerometer (Not TeleNano/TeleMini)
    pres: i16, // Pressure Sensor
    temp: i16, // Temperature Sensor
    v_batt: i16, // Battery Voltage
    sense_d: i16, // Drogue Continuity Sense (Not TeleNano)
    sense_m: i16, // Main Continuity (Not TeleNano)
    acceleration: i16, // m/s^2 * 16
    speed: i16, // m/s * 16
    height: i16, // m
    ground_pres: i16, // Average Barometer Reading on Ground 
    ground_accel: i16, // (Not TeleNano/TeleMini)
    accel_plus_g: i16, // (Not TeleNano/TeleMini)
    accel_minus_g: i16 // (Not TeleNano/TeleMini)
}

impl PayloadDecode for Sensor {
    fn decode(type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Self, DecodeError> { todo!() }
}

#[derive(Debug, Clone)]
pub struct MegaImu {
    orient: u8, // Angle from Vertical (degrees)
    accel: i16, // High G Accelerometer
    pres: i32, // Pressure (Pa * 10)
    temp: i16, // Temperature (°C * 100)
    accel_x: i16, // X Axis Acceleration (across)
    accel_y: i16, // Y Axis Acceleration (Along)
    accel_z: i16, // Z Axis Acceleration (Through)
    gyro_x: i16, // X Axis Rotation (Roll)
    gyro_y: i16, // Y Axis Rotation (Pitch)
    gyro_z: i16, // Z Axis Rotation (Yaw)
    mag_x: i16, // X Field Strength (Across)
    mag_y: i16, // Y Field Strength (Along)
    mag_z: i16, // Z Field Strength (Through)
}

impl PayloadDecode for MegaImu {
    fn decode(type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Self, DecodeError> { todo!() }
}

#[derive(Debug, Clone)]
pub struct MegaKalman {
    state: u8, // Flight State
    v_batt: i16, // Battery Voltage
    v_pyro: i16, // Pyro Battery Voltage
    sense: [i8; 6], // Pyro Continuity Sense
    ground_pres: i32, // Average Barometer Reading on Ground
    ground_accel: i16, // Average Accelerometer Reading on Ground
    accel_plus_g: i16, // Accel Calibration at +1g
    accel_minus_g: i16, // Accel Calibration at -1g
    acceleration: i16, // m/s^2 * 16
    speed: i16, // m/s * 16
    height: i16, // m
}

impl PayloadDecode for MegaKalman {
    fn decode(type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Self, DecodeError> { todo!() }
}

#[derive(Debug, Clone)]
pub struct MetrumSensor {
    state: u8, // Flight State
    accel: i16, // Accelerometer
    pres: i32, // Pressure Sensor (Pa * 10)
    temp: i16, // Temperature Sensor (°C * 100)
    acceleration: i16, // m/s^2 * 16
    speed: i16, // m/s * 16
    height: i16, // m
    v_batt: i16, // Battery Voltage
    sense_d: i16, // Drogue Continuity Sense
    sense_m: i16, // Main Continuity Sense
    // 6 Bytes of padding
}

impl PayloadDecode for MetrumSensor {
    fn decode(type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Self, DecodeError> { todo!() }
}

#[derive(Debug, Clone)]
pub struct MetrumCalibration {
    // 3 bytes of padding
    ground_pres: i32, // Average Barometer Reading on Ground
    ground_accel: i16, // Average Accelerometer Reading on Ground
    accel_plus_g: i16, // Accel calibration at +1g
    accel_minus_g: i16, // Accel calibration at -1g
    // 14 bytes of padding
}

impl PayloadDecode for MetrumCalibration {
    fn decode(type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Self, DecodeError> { todo!() }
}

#[derive(Debug, Clone)]
pub struct MiniSensor {
    state: u8, // Flight State
    v_batt: i16, // Battery Voltage
    sense_a: i16, // Apogee Continuity Sense
    sense_m: i16, // Main Continuity Sense
    pres: i32, // Pressure Sensor (Pa * 10)
    temp: i16, // Temperature Sensor (°C * 100)
    acceleration: i16, // m/s^2 * 16
    speed: i16, // m/s * 16
    height: i16, // m
    ground_pres: i32, // Average Barometer Reading on Ground
    // 4 bytes of padding
}

impl PayloadDecode for MiniSensor {
    fn decode(type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Self, DecodeError> { todo!() }
}

#[derive(Debug, Clone)]
pub struct Config {
    dev_type: u8, // Device Type
    flight: u16, // Flight Number
    config_major: u8, // Config Major Version
    config_minor: u8, // Config Minor Version
    apogee_delay: u16, // Apogee Deploy Delay (seconds)
    main_deploy: u16, // Main Deploy Alt (meters)
    flight_log_max: u16, // Maximum Flight Log Size (kB)
    callsign: [u8; 8], // Radio Operator Identifier (chars)
    version: [u8; 8], // Software Version Identifier (chars)
    // C++'s characters are 1 byte while Rust's are 4
    // Using u8 since it's 1 byte to get around this
}

impl PayloadDecode for Config {
    fn decode(type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Self, DecodeError> { todo!() }
}

#[derive(Debug, Clone)]
pub struct Companion {
    board_id: u8, // Type of Companion Board Attached
    update_period: u8, // How often Telemetry is sent (1/100ths of a second)
    channels: u8, // Number of Data Channels Supplied
    companion_data: [u16; 12], // Up to 12 channels of 16-bit Companion Data
}

impl PayloadDecode for Companion {
    fn decode(type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Self, DecodeError> { todo!() }
}

#[derive(Debug)]
pub enum DecodeError {
    BadPrefix,                         // line doesn't start with "TELEM "
    BadHex,
    BadLength { expected: usize, got: usize },
    BadChecksum { expected: u8, got: u8 },
    CrcFailed,                         // LQI bit 7 clear: radio says corrupt
    Truncated,                         // payload shorter than its layout needs
    BadPacketLength,
    BadPayloadLength,
    ConversionError,
    UnknownDataType
}

/// line -> framing check -> header + payload bytes -> typed payload
pub fn parse_line(line: &str) -> Result<Telemetry, DecodeError> { 
    let start = "TELEM ";
    if !line.starts_with(start) {
        return Err(DecodeError::BadPrefix);
    }

    let line = line.trim_start_matches(start);
    let bytes = hex_to_bytes(&line)?; 

    // Bytes is 32 long (includes header)
    let (bytes, link) = verify_frame(&bytes)?;

    // 2 bytes of Serial, 2 bytes of Tick
    let type_id: u8 = bytes[4];

    // 27 bytes of payload data
    let (bytes, header) = extract_headers(&bytes)?;

    // Returns the formatted version of the payload data
    let payload = decode_payload(type_id, &bytes)?;

    let resp = Telemetry {
        header: header,
        link: link,
        payload: payload
    };

    Ok(resp)
}

// Convert the entire line into a Vec of bytes
fn hex_to_bytes(s: &str) -> Result<Vec<u8>, DecodeError> { 
    // Shouldn't start with it, but just in case it does
    let s = s.trim_start_matches("0x");

    hex::decode(s).map_err(|_| DecodeError::BadHex)
}

// Pull out RSSI & LQI information, Verify Length is correct, verify Checksum is correct
fn verify_frame(bytes: &[u8]) -> Result<([u8; PACKET_LEN], LinkInfo), DecodeError> { 
    let given_len: usize = bytes[0].try_into().map_err(|_| DecodeError::BadHex)?; // 34
    let act_len = bytes.len(); // 36 (Includes length and sum bytes)

    // Lost bytes somewhere in transit, Given length isn't the same as its actual length
    // +2 because given_len does not include length and sum bytes, while the actual length does
    if given_len + 2 != act_len {
        return Err(DecodeError::BadLength { expected: given_len + 2, got: act_len })
    }

    // Given length is accurate, but not the length that is expected 
    // +2 because PACKET_LEN does not include RSSI & LQI data while given_len does
    if given_len != PACKET_LEN + 2 {
        return Err(DecodeError::BadLength { expected: PACKET_LEN, got: given_len });
    }

    check_sum(bytes)?;

    let lqi = bytes[act_len - 2]; // 1 for checksum & one for going from 1 based to 0 based
    let rssi = bytes[act_len - 3];
    let crc_ok = lqi & (1 << 6) == 1; // If the 7th bit is on (AND with 0100 0000) 

    if !crc_ok {
        return Err(DecodeError::CrcFailed);
    }

    let link = LinkInfo {
        lqi: lqi,
        rssi_dbm: f32::from(rssi) / 2.0 - 74.0,
        crc_ok: crc_ok
    };

    let packet = &bytes[1..=PACKET_LEN];
    let packet: [u8; PACKET_LEN] = packet.try_into().map_err(|_| DecodeError::BadPacketLength)?;

    Ok((packet, link))
}

fn check_sum(bytes: &[u8]) -> Result<(), DecodeError> {
    let len = bytes.len();
    let mut sum: i32 = 0x5a;

    for b in bytes {
        sum += i32::from(*b);
    }
    sum &= 0xff; // Get the last 8 bits
    let sum: u8 = sum.try_into().map_err(|_| DecodeError::ConversionError)?;

    if !(sum == bytes[len - 1]) {
        return Err(DecodeError::BadChecksum { expected: bytes[len - 1], got: sum });
    }

    Ok(())
}

// Pull out header bytes from the payload bytes
fn extract_headers(bytes: &[u8; PACKET_LEN]) -> Result<([u8; PAYLOAD_LEN], Header), DecodeError> { 
    // Take the byte on the left, shift its bits by 4, then add to byte on the right

    let serial = u16::from(bytes[0]) << 4;
    let serial =  serial + u16::from(bytes[1]);
    
    let tick = u16::from(bytes[2]) << 4;
    let tick = tick + u16::from(bytes[3]);

    let telem_type = bytes[4];

    let header = Header {
        serial: serial,
        tick: tick,
        type_id: telem_type
    };

    let resp: [u8; PAYLOAD_LEN] = bytes[5..bytes.len()].try_into().map_err(|_| DecodeError::BadPayloadLength)?;
    Ok((resp, header))
}

// Convert the line into its Payload type
fn decode_payload(type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Payload, DecodeError> { 
    let payload_type = match type_id {
        0x01 | 0x02 | 0x03          =>  Payload::Sensor(Sensor::decode(type_id, raw)?),
        0x04                        =>  Payload::Config(Config::decode(type_id, raw)?),
        0x05                        =>  Payload::Location(Location::decode(type_id, raw)?),
        0x06                        =>  Payload::Satellites(Satellites::decode(type_id, raw)?),
        0x07                        =>  Payload::Companion(Companion::decode(type_id, raw)?),
        0x08 | 0x12 | 0x13 | 0x14   =>  Payload::MegaImu(MegaImu::decode(type_id, raw)?),
        0x09 | 0x15                 =>  Payload::MegaKalman(MegaKalman::decode(type_id, raw)?),
        0x11                        =>  Payload::MiniSensor(MiniSensor::decode(type_id, raw)?),
        0x0A                        =>  Payload::MetrumSensor(MetrumSensor::decode(type_id, raw)?),
        0x0B                        =>  Payload::MetrumCalibration(MetrumCalibration::decode(type_id, raw)?),

        _ => Payload::Unknown { type_id: type_id, raw: *raw }
    };

    if matches!(payload_type, Payload::Unknown { .. }) {
        return Err(DecodeError::UnknownDataType);
    }

    Ok(payload_type)
}