pub const PACKET_LEN: usize = 32; // Does not include length byte or checksum byte
pub const PAYLOAD_LEN: usize = PACKET_LEN - 5; // 27 data bytes

// Struct to help with reading bytes in the array
pub struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    /// Take the next N bytes and advance
    fn take<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        let end = self.pos + N;
        let bytes = self.buf.get(self.pos..end).ok_or(DecodeError::Truncated)?;
        self.pos = end;
        Ok(bytes.try_into().unwrap()) // safe: slice length is exactly N
    }

    pub fn u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.take::<1>()?[0])
    }
    pub fn i8(&mut self) -> Result<i8, DecodeError> {
        Ok(self.take::<1>()?[0] as i8)
    }
    pub fn u16(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_le_bytes(self.take()?))
    }
    pub fn i16(&mut self) -> Result<i16, DecodeError> {
        Ok(i16::from_le_bytes(self.take()?))
    }
    pub fn i32(&mut self) -> Result<i32, DecodeError> {
        Ok(i32::from_le_bytes(self.take()?))
    }

    /// Skip padding bytes
    pub fn skip(&mut self, n: usize) -> Result<(), DecodeError> {
        if self.pos + n > self.buf.len() {
            return Err(DecodeError::Truncated);
        }
        self.pos += n;
        Ok(())
    }

    pub fn finish(&self) {
        debug_assert_eq!(self.pos, self.buf.len())
    }
    pub fn bytes<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        self.take()
    }
}

/// What the TeleDongle appended/prepended (not part of the radio packet)
#[derive(Debug, Clone, Copy)]
pub struct LinkInfo {
    pub rssi_dbm: f32, // derived from raw i8: raw as f32 / 2.0 - 74.0
    pub lqi: u8,
    pub crc_ok: bool, // bit 7 of the raw LQI byte
}

/// Common to every packet
#[derive(Debug, Clone, Copy)]
pub struct Header {
    pub serial: u16, // Serial Number of Transmitting Device
    pub tick: u16,   // Current Relative Timestamp (1/100ths of a second)
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

pub trait PayloadDecode: Sized {
    fn decode(type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Self, DecodeError>;
}

#[derive(Debug, Clone)]
pub struct Location {
    flags: GpsFlags, // Based on the first byte of information
    altitude: i16,   // m
    latitude: i32,   // degrees * 10^7
    longitude: i32,  // degrees * 10^7
    year: u8,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    pdop: u8,      // * 5
    hdop: u8,      // * 5
    vdop: u8,      // * 5
    mode: GpsMode, // 1 byte
    ground_speed: u16,
    climb_rate: i16,
    course: u8,
    // 1 Byte of padding
}

impl PayloadDecode for Location {
    fn decode(_type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Self, DecodeError> {
        let mut c = Cursor::new(raw);

        let resp = Location {
            flags: GpsFlags::from_byte(c.u8()?),
            altitude: c.i16()?,
            latitude: c.i32()?,
            longitude: c.i32()?,
            year: c.u8()?,
            month: c.u8()?,
            day: c.u8()?,
            hour: c.u8()?,
            minute: c.u8()?,
            second: c.u8()?,
            pdop: c.u8()?,
            hdop: c.u8()?,
            vdop: c.u8()?,
            mode: GpsMode::from_byte(c.u8()?),
            ground_speed: c.u16()?,
            climb_rate: c.i16()?,
            course: c.u8()?,
        };

        c.skip(1)?;
        c.finish();

        Ok(resp)
    }
}

#[derive(Debug, Clone)]
pub struct GpsFlags {
    nsats: u8,          // Bits 0-3, Number of Satellites in Solution
    valid: bool,        // Bit 4, GPS Solution is Valid
    running: bool,      // Bit 5, GPS Receiver is Operational
    date_valid: bool,   // Bit 6, Reported Date is valid
    course_valid: bool, // Bit 7, Ground Speed, Course, & Climb Rates are Valid
}

impl GpsFlags {
    fn from_byte(b: u8) -> Self {
        Self {
            nsats: b & 0x0f,                 // AND with 0000 1111 to get first 4
            valid: b & (1 << 4) != 0,        // AND with 0001 0000
            running: b & (1 << 5) != 0,      // AND with 0010 0000
            date_valid: b & (1 << 6) != 0,   // AND with 0100 0000
            course_valid: b & (1 << 7) != 0, // AND with 1000 0000
        }
    }
}

#[derive(Debug, Clone)]
pub enum GpsMode {
    NotValid,     // All Data is Invalid
    Autonomous,   // Data is Derived from Satellite Data
    Differential, // Data is Augmented with Differential Data from a Known Ground Station
    Estimated,    // Data is Estimated using Dead Reckoning from the Last Known Data
    Manual,       // Data was Entered Manually
    Simulated,    // GPS Receiver Testing Mode
    Other(u8),
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
            other => Self::Other(other),
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
    fn decode(_type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Self, DecodeError> {
        let mut c = Cursor::new(raw);

        let channels = c.u8()?;
        let mut sats = [SatelliteInfo::default(); 12];
        for s in sats.iter_mut() {
            *s = SatelliteInfo::new(c.u8()?, c.u8()?);
        }

        let resp = Satellites {
            channels: channels,
            sats: sats,
        };

        c.skip(2)?;
        c.finish();

        Ok(resp)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SatelliteInfo {
    svid: u8,  // Space Vehicle Identifier
    c_n_1: u8, // C/N1 signal quality indicator
}

impl SatelliteInfo {
    pub fn new(svid: u8, c_n_1: u8) -> Self {
        SatelliteInfo {
            svid: svid,
            c_n_1: c_n_1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sensor {
    state: u8,          // Flight State
    accel: i16,         // Accelerometer (Not TeleNano/TeleMini)
    pres: i16,          // Pressure Sensor
    temp: i16,          // Temperature Sensor
    v_batt: i16,        // Battery Voltage
    sense_d: i16,       // Drogue Continuity Sense (Not TeleNano)
    sense_m: i16,       // Main Continuity (Not TeleNano)
    acceleration: i16,  // m/s^2 * 16
    speed: i16,         // m/s * 16
    height: i16,        // m
    ground_pres: i16,   // Average Barometer Reading on Ground
    ground_accel: i16,  // (Not TeleNano/TeleMini)
    accel_plus_g: i16,  // (Not TeleNano/TeleMini)
    accel_minus_g: i16, // (Not TeleNano/TeleMini)
    kind: SensorKind,
}

impl PayloadDecode for Sensor {
    fn decode(type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Self, DecodeError> {
        let mut c = Cursor::new(raw);

        let kind = match type_id {
            0x01 => SensorKind::TeleMetrumV1,
            0x02 => SensorKind::TeleMiniV1,
            0x03 => SensorKind::TeleNano,
            _ => return Err(DecodeError::InvalidTypeId),
        };

        let resp = Sensor {
            state: c.u8()?,
            accel: c.i16()?,
            pres: c.i16()?,
            temp: c.i16()?,
            v_batt: c.i16()?,
            sense_d: c.i16()?,
            sense_m: c.i16()?,
            acceleration: c.i16()?,
            speed: c.i16()?,
            height: c.i16()?,
            ground_pres: c.i16()?,
            ground_accel: c.i16()?,
            accel_plus_g: c.i16()?,
            accel_minus_g: c.i16()?,
            kind: kind,
        };

        c.finish();

        Ok(resp)
    }
}

#[derive(Debug, Clone)]
pub enum SensorKind {
    TeleMetrumV1,
    TeleMiniV1,
    TeleNano,
}
#[derive(Debug, Clone)]
pub struct MegaImu {
    orient: u8,   // Angle from Vertical (degrees)
    accel: i16,   // High G Accelerometer
    pres: i32,    // Pressure (Pa * 10)
    temp: i16,    // Temperature (°C * 100)
    accel_x: i16, // X Axis Acceleration (across)
    accel_y: i16, // Y Axis Acceleration (Along)
    accel_z: i16, // Z Axis Acceleration (Through)
    gyro_x: i16,  // X Axis Rotation (Roll)
    gyro_y: i16,  // Y Axis Rotation (Pitch)
    gyro_z: i16,  // Z Axis Rotation (Yaw)
    mag_x: i16,   // X Field Strength (Across)
    mag_y: i16,   // Y Field Strength (Along)
    mag_z: i16,   // Z Field Strength (Through)
    kind: ImuKind,
}

#[derive(Debug, Clone)]
pub enum ImuKind {
    Invensense,
    BMX160,
    MPU6000,
    BMI088,
}

impl PayloadDecode for MegaImu {
    fn decode(type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Self, DecodeError> {
        let mut c = Cursor::new(raw);

        let kind = match type_id {
            0x08 => ImuKind::Invensense,
            0x12 => ImuKind::BMX160,
            0x13 => ImuKind::MPU6000,
            0x14 => ImuKind::BMI088,
            _ => return Err(DecodeError::InvalidTypeId),
        };

        let resp = MegaImu {
            orient: c.u8()?,
            accel: c.i16()?,
            pres: c.i32()?,
            temp: c.i16()?,
            accel_x: c.i16()?,
            accel_y: c.i16()?,
            accel_z: c.i16()?,
            gyro_x: c.i16()?,
            gyro_y: c.i16()?,
            gyro_z: c.i16()?,
            mag_x: c.i16()?,
            mag_y: c.i16()?,
            mag_z: c.i16()?,
            kind: kind,
        };

        c.finish();

        Ok(resp)
    }
}

#[derive(Debug, Clone)]
pub struct MegaKalman {
    state: u8,          // Flight State
    v_batt: i16,        // Battery Voltage
    v_pyro: i16,        // Pyro Battery Voltage
    sense: [u8; 6],     // Pyro Continuity Sense
    ground_pres: i32,   // Average Barometer Reading on Ground
    ground_accel: i16,  // Average Accelerometer Reading on Ground
    accel_plus_g: i16,  // Accel Calibration at +1g
    accel_minus_g: i16, // Accel Calibration at -1g
    acceleration: i16,  // m/s^2 * 16
    speed: i16,         // m/s * 16
    height: i16,        // m
    voltage: VoltageRange,
}

#[derive(Debug, Clone)]
pub enum VoltageRange {
    V15,
    V30,
}

impl PayloadDecode for MegaKalman {
    fn decode(type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Self, DecodeError> {
        let mut c = Cursor::new(raw);

        let voltage = match type_id {
            0x09 => VoltageRange::V15,
            0x15 => VoltageRange::V30,
            _ => return Err(DecodeError::InvalidTypeId),
        };

        let resp = MegaKalman {
            state: c.u8()?,
            v_batt: c.i16()?,
            v_pyro: c.i16()?,
            sense: c.bytes::<6>()?,
            ground_pres: c.i32()?,
            ground_accel: c.i16()?,
            accel_plus_g: c.i16()?,
            accel_minus_g: c.i16()?,
            acceleration: c.i16()?,
            speed: c.i16()?,
            height: c.i16()?,
            voltage: voltage,
        };

        c.finish();

        Ok(resp)
    }
}

#[derive(Debug, Clone)]
pub struct MetrumSensor {
    state: u8,         // Flight State
    accel: i16,        // Accelerometer
    pres: i32,         // Pressure Sensor (Pa * 10)
    temp: i16,         // Temperature Sensor (°C * 100)
    acceleration: i16, // m/s^2 * 16
    speed: i16,        // m/s * 16
    height: i16,       // m
    v_batt: i16,       // Battery Voltage
    sense_d: i16,      // Drogue Continuity Sense
    sense_m: i16,      // Main Continuity Sense
                       // 6 Bytes of padding
}

impl PayloadDecode for MetrumSensor {
    fn decode(_type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Self, DecodeError> {
        let mut c = Cursor::new(raw);

        let resp = MetrumSensor {
            state: c.u8()?,
            accel: c.i16()?,
            pres: c.i32()?,
            temp: c.i16()?,
            acceleration: c.i16()?,
            speed: c.i16()?,
            height: c.i16()?,
            v_batt: c.i16()?,
            sense_d: c.i16()?,
            sense_m: c.i16()?,
        };

        c.skip(6)?;
        c.finish();

        Ok(resp)
    }
}

#[derive(Debug, Clone)]
pub struct MetrumCalibration {
    // 3 bytes of padding
    ground_pres: i32,  // Average Barometer Reading on Ground
    ground_accel: i16, // Average Accelerometer Reading on Ground
    accel_plus_g: i16, // Accel calibration at +1g
    accel_minus_g: i16, // Accel calibration at -1g
                       // 14 bytes of padding
}

impl PayloadDecode for MetrumCalibration {
    fn decode(_type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Self, DecodeError> {
        let mut c = Cursor::new(raw);

        c.skip(3)?;

        let resp = MetrumCalibration {
            ground_pres: c.i32()?,
            ground_accel: c.i16()?,
            accel_plus_g: c.i16()?,
            accel_minus_g: c.i16()?,
        };

        c.skip(14)?;
        c.finish();

        Ok(resp)
    }
}

#[derive(Debug, Clone)]
pub struct MiniSensor {
    state: u8,         // Flight State
    v_batt: i16,       // Battery Voltage
    sense_a: i16,      // Apogee Continuity Sense
    sense_m: i16,      // Main Continuity Sense
    pres: i32,         // Pressure Sensor (Pa * 10)
    temp: i16,         // Temperature Sensor (°C * 100)
    acceleration: i16, // m/s^2 * 16
    speed: i16,        // m/s * 16
    height: i16,       // m
    ground_pres: i32,  // Average Barometer Reading on Ground
                       // 4 bytes of padding
}

impl PayloadDecode for MiniSensor {
    fn decode(_type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Self, DecodeError> {
        let mut c = Cursor::new(raw);

        let resp = MiniSensor {
            state: c.u8()?,
            v_batt: c.i16()?,
            sense_a: c.i16()?,
            sense_m: c.i16()?,
            pres: c.i32()?,
            temp: c.i16()?,
            acceleration: c.i16()?,
            speed: c.i16()?,
            height: c.i16()?,
            ground_pres: c.i32()?,
        };

        c.skip(4)?;
        c.finish();

        Ok(resp)
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    dev_type: u8,        // Device Type
    flight: u16,         // Flight Number
    config_major: u8,    // Config Major Version
    config_minor: u8,    // Config Minor Version
    apogee_delay: u16,   // Apogee Deploy Delay (seconds)
    main_deploy: u16,    // Main Deploy Alt (meters)
    flight_log_max: u16, // Maximum Flight Log Size (kB)
    callsign: [u8; 8],   // Radio Operator Identifier (chars)
    version: [u8; 8],    // Software Version Identifier (chars)
                         // C++'s characters are 1 byte while Rust's are 4
                         // Using u8 since it's 1 byte to get around this
}

impl PayloadDecode for Config {
    fn decode(_type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Self, DecodeError> {
        let mut c = Cursor::new(raw);

        let resp = Config {
            dev_type: c.u8()?,
            flight: c.u16()?,
            config_major: c.u8()?,
            config_minor: c.u8()?,
            apogee_delay: c.u16()?,
            main_deploy: c.u16()?,
            flight_log_max: c.u16()?,
            callsign: c.bytes::<8>()?,
            version: c.bytes::<8>()?,
        };

        c.finish();

        Ok(resp)
    }
}

#[derive(Debug, Clone)]
pub struct Companion {
    board_id: u8,              // Type of Companion Board Attached
    update_period: u8,         // How often Telemetry is sent (1/100ths of a second)
    channels: u8,              // Number of Data Channels Supplied
    companion_data: [u16; 12], // Up to 12 channels of 16-bit Companion Data
}

impl PayloadDecode for Companion {
    fn decode(_type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Self, DecodeError> {
        let mut c = Cursor::new(raw);

        let (board_id, update_period, channels) = (c.u8()?, c.u8()?, c.u8()?);

        let mut companion_data = [0u16; 12];
        for d in companion_data.iter_mut() {
            *d = c.u16()?;
        }

        let resp = Companion {
            board_id: board_id,
            update_period: update_period,
            channels: channels,
            companion_data: companion_data,
        };

        c.finish();

        Ok(resp)
    }
}

#[derive(Debug)]
pub enum DecodeError {
    BadPrefix, // line doesn't start with "TELEM "
    BadHex,
    BadLength { expected: usize, got: usize },
    BadChecksum { expected: u8, got: u8 },
    CrcFailed, // LQI bit 7 clear: radio says corrupt
    Truncated, // payload shorter than its layout needs
    BadPacketLength,
    BadPayloadLength,
    ConversionError,
    InvalidTypeId,
}

// line -> framing check -> header + payload bytes -> typed payload
// The only function that should be called from outside this file
pub fn parse_line(line: &str) -> Result<Telemetry, DecodeError> {
    let start = "TELEM ";
    let line = line
        .trim()
        .strip_prefix(start)
        .ok_or(DecodeError::BadPrefix)?;

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
        payload: payload,
    };

    Ok(resp)
}

// Convert the entire line into a Vec of bytes
fn hex_to_bytes(s: &str) -> Result<Vec<u8>, DecodeError> {
    hex::decode(s).map_err(|_| DecodeError::BadHex)
}

// Pull out RSSI & LQI information, Verify Length is correct, verify Checksum is correct
fn verify_frame(bytes: &[u8]) -> Result<([u8; PACKET_LEN], LinkInfo), DecodeError> {
    let given_len: u8 = *bytes.first().ok_or(DecodeError::BadPayloadLength)?;
    let given_len: usize = given_len.try_into().map_err(|_| DecodeError::BadHex)?; // 34
    let act_len = bytes.len(); // 36 (Includes length and sum bytes)

    // Lost bytes somewhere in transit, Given length isn't the same as its actual length
    // +2 because given_len does not include length and sum bytes, while the actual length does
    if given_len + 2 != act_len {
        return Err(DecodeError::BadLength {
            expected: given_len + 2,
            got: act_len,
        });
    }

    // Given length is accurate, but not the length that is expected
    // +2 because PACKET_LEN does not include RSSI & LQI data while given_len does
    if given_len != PACKET_LEN + 2 {
        return Err(DecodeError::BadLength {
            expected: PACKET_LEN,
            got: given_len,
        });
    }

    check_sum(bytes)?;

    let lqi = bytes[act_len - 2]; // 1 for checksum & one for going from 1 based to 0 based
    let rssi = bytes[act_len - 3] as i8;
    let crc_ok = lqi & (1 << 7) != 0; // If the 8th bit (bit 7) is on (AND with 1000 0000)

    if !crc_ok {
        return Err(DecodeError::CrcFailed);
    }

    let link = LinkInfo {
        lqi: lqi,
        rssi_dbm: f32::from(rssi) / 2.0 - 74.0,
        crc_ok: crc_ok,
    };

    let packet = &bytes[1..=PACKET_LEN];
    let packet: [u8; PACKET_LEN] = packet
        .try_into()
        .map_err(|_| DecodeError::BadPacketLength)?;

    Ok((packet, link))
}

fn check_sum(bytes: &[u8]) -> Result<(), DecodeError> {
    let len = bytes.len();
    let mut sum: i32 = 0x5a;

    let (&expected, body) = bytes.split_last().ok_or(DecodeError::Truncated)?;

    // Skip the length byte
    for b in body[1..].iter() {
        sum += i32::from(*b);
    }
    sum &= 0xff; // Get the last 8 bits
    let sum: u8 = sum.try_into().map_err(|_| DecodeError::ConversionError)?;

    if !(sum == expected) {
        return Err(DecodeError::BadChecksum {
            expected: bytes[len - 1],
            got: sum,
        });
    }

    Ok(())
}

// Pull out header bytes from the payload bytes
fn extract_headers(bytes: &[u8; PACKET_LEN]) -> Result<([u8; PAYLOAD_LEN], Header), DecodeError> {
    // Take the byte on the left, shift its bits by 8, then add to byte on the right

    let serial = u16::from_le_bytes([bytes[0], bytes[1]]);
    // let serial = u16::from(bytes[0]) << 8;
    // let serial =  serial + u16::from(bytes[1]);

    let tick = u16::from_le_bytes([bytes[2], bytes[3]]);
    // let tick = u16::from(bytes[2]) << 8;
    // let tick = tick + u16::from(bytes[3]);

    let telem_type = bytes[4];

    let header = Header {
        serial: serial,
        tick: tick,
        type_id: telem_type,
    };

    let resp: [u8; PAYLOAD_LEN] = bytes[5..bytes.len()]
        .try_into()
        .map_err(|_| DecodeError::BadPayloadLength)?;
    Ok((resp, header))
}

// Convert the line into its Payload type
fn decode_payload(type_id: u8, raw: &[u8; PAYLOAD_LEN]) -> Result<Payload, DecodeError> {
    let payload_type = match type_id {
        0x01 | 0x02 | 0x03 => Payload::Sensor(Sensor::decode(type_id, raw)?),
        0x04 => Payload::Config(Config::decode(type_id, raw)?),
        0x05 => Payload::Location(Location::decode(type_id, raw)?),
        0x06 => Payload::Satellites(Satellites::decode(type_id, raw)?),
        0x07 => Payload::Companion(Companion::decode(type_id, raw)?),
        0x08 | 0x12 | 0x13 | 0x14 => Payload::MegaImu(MegaImu::decode(type_id, raw)?),
        0x09 | 0x15 => Payload::MegaKalman(MegaKalman::decode(type_id, raw)?),
        0x11 => Payload::MiniSensor(MiniSensor::decode(type_id, raw)?),
        0x0A => Payload::MetrumSensor(MetrumSensor::decode(type_id, raw)?),
        0x0B => Payload::MetrumCalibration(MetrumCalibration::decode(type_id, raw)?),

        _ => Payload::Unknown {
            type_id: type_id,
            raw: *raw,
        },
    };

    Ok(payload_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ───────────────────────── helpers ─────────────────────────

    /// Payload builder. build() insists on exactly 27 bytes, so a wrong
    /// layout in a test fails loudly instead of silently shifting fields.
    #[derive(Default)]
    struct Pb(Vec<u8>);
    impl Pb {
        fn u8(mut self, v: u8) -> Self {
            self.0.push(v);
            self
        }
        fn u16(mut self, v: u16) -> Self {
            self.0.extend_from_slice(&v.to_le_bytes());
            self
        }
        fn i16(mut self, v: i16) -> Self {
            self.0.extend_from_slice(&v.to_le_bytes());
            self
        }
        fn i32(mut self, v: i32) -> Self {
            self.0.extend_from_slice(&v.to_le_bytes());
            self
        }
        fn raw(mut self, b: &[u8]) -> Self {
            self.0.extend_from_slice(b);
            self
        }
        fn fill(mut self, n: usize, v: u8) -> Self {
            self.0.extend(std::iter::repeat(v).take(n));
            self
        }
        fn build(self) -> [u8; PAYLOAD_LEN] {
            self.0.try_into().expect("payload must be exactly 27 bytes")
        }
    }

    fn packet(
        serial: u16,
        tick: u16,
        type_id: u8,
        payload: &[u8; PAYLOAD_LEN],
    ) -> [u8; PACKET_LEN] {
        let mut p = [0u8; PACKET_LEN];
        p[0..2].copy_from_slice(&serial.to_le_bytes());
        p[2..4].copy_from_slice(&tick.to_le_bytes());
        p[4] = type_id;
        p[5..].copy_from_slice(payload);
        p
    }

    /// length byte + packet + rssi + lqi + checksum
    fn frame(pkt: &[u8; PACKET_LEN], rssi: i8, lqi: u8) -> Vec<u8> {
        let mut v = vec![(PACKET_LEN + 2) as u8];
        v.extend_from_slice(pkt);
        v.push(rssi as u8);
        v.push(lqi);
        v.push(checksum_of(&v));
        v
    }

    fn checksum_of(bytes_without_checksum: &[u8]) -> u8 {
        bytes_without_checksum[1..]
            .iter()
            .fold(0x5a_u8, |a, &b| a.wrapping_add(b))
    }

    fn line(bytes: &[u8]) -> String {
        format!("TELEM {}", hex::encode(bytes))
    }

    /// A valid frame for the given type, parsed.
    fn parse_ok(type_id: u8, payload: &[u8; PAYLOAD_LEN]) -> Telemetry {
        parse_line(&line(&frame(
            &packet(0x1234, 100, type_id, payload),
            -100,
            0x91,
        )))
        .unwrap()
    }

    fn good_bytes() -> Vec<u8> {
        frame(&packet(0x1234, 100, 0x7f, &[7u8; PAYLOAD_LEN]), -100, 0x91)
    }

    fn variant_name(p: &Payload) -> &'static str {
        match p {
            Payload::Sensor(_) => "Sensor",
            Payload::Config(_) => "Config",
            Payload::Location(_) => "Location",
            Payload::Satellites(_) => "Satellites",
            Payload::Companion(_) => "Companion",
            Payload::MegaImu(_) => "MegaImu",
            Payload::MegaKalman(_) => "MegaKalman",
            Payload::MiniSensor(_) => "MiniSensor",
            Payload::MetrumSensor(_) => "MetrumSensor",
            Payload::MetrumCalibration(_) => "MetrumCalibration",
            Payload::Unknown { .. } => "Unknown",
        }
    }

    // ───────────────────── real captured lines ─────────────────────

    #[test]
    fn real_gps_line() {
        let t = parse_line(
            "TELEM 22fe3f1b0805ff7b033d788112c4024bc21a061300192f09040801030000007d00d091c9",
        )
        .unwrap();
        assert_eq!(t.header.serial, 16382);
        assert_eq!(t.header.tick, 2075);
        assert_eq!(t.header.type_id, 0x05);
        assert_eq!(t.link.rssi_dbm, -98.0);
        assert_eq!(t.link.lqi, 0x91); // raw byte, bit 7 included
        assert!(t.link.crc_ok);

        let Payload::Location(l) = t.payload else {
            panic!("wrong type")
        };
        assert_eq!(l.flags.nsats, 15);
        assert!(l.flags.valid && l.flags.running && l.flags.date_valid && l.flags.course_valid);
        assert_eq!(l.altitude, 891);
        assert_eq!(l.latitude, 310_474_813);
        assert_eq!(l.longitude, -1_035_271_484);
        assert_eq!((l.year, l.month, l.day), (26, 6, 19));
        assert_eq!((l.hour, l.minute, l.second), (0, 25, 47));
        assert_eq!((l.pdop, l.hdop, l.vdop), (9, 4, 8));
        assert!(matches!(l.mode, GpsMode::Other(1)));
        assert_eq!(l.ground_speed, 3);
        assert_eq!(l.climb_rate, 0);
        assert_eq!(l.course, 125);
    }

    #[test]
    fn real_config_line() {
        let t = parse_line(
            "TELEM 22fe3f2c3004230100011b0000310100044b45394552490000312e392e32320000de88a6",
        )
        .unwrap();
        assert_eq!(t.link.rssi_dbm, -91.0);
        assert_eq!(t.link.lqi, 0x88);

        let Payload::Config(c) = t.payload else {
            panic!("wrong type")
        };
        assert_eq!(c.dev_type, 0x23);
        assert_eq!(c.flight, 1);
        assert_eq!((c.config_major, c.config_minor), (1, 27));
        assert_eq!(c.apogee_delay, 0);
        assert_eq!(c.main_deploy, 305);
        assert_eq!(c.flight_log_max, 1024);
        assert_eq!(c.callsign, *b"KE9ERI\0\0");
        assert_eq!(c.version, *b"1.9.22\0\0");
    }

    // ───────────────────── line / framing edge cases ─────────────────────

    #[test]
    fn prefix_errors() {
        let hexpart = hex::encode(good_bytes());
        for bad in [
            hexpart.clone(),
            format!("telem {hexpart}"),
            format!("TELEM{hexpart}"),
            "TELEM ".to_string(),
            "TELEM".to_string(),
            String::new(),
        ] {
            assert!(
                matches!(parse_line(&bad), Err(DecodeError::BadPrefix)),
                "input: {bad:?}"
            );
        }
    }

    #[test]
    fn surrounding_whitespace_and_hex_case_are_accepted() {
        let l = line(&good_bytes());
        assert!(parse_line(&format!("{l}\n")).is_ok());
        assert!(parse_line(&format!("{l}\r\n")).is_ok());
        assert!(parse_line(&format!("   {l}  ")).is_ok());
        assert!(parse_line(&l.to_uppercase()).is_ok());
    }

    #[test]
    fn bad_hex() {
        assert!(matches!(parse_line("TELEM abc"), Err(DecodeError::BadHex))); // odd length
        assert!(matches!(parse_line("TELEM zz"), Err(DecodeError::BadHex))); // not hex
        let l = line(&good_bytes());
        assert!(matches!(
            parse_line(&format!("{l} ")[..l.len() - 1].replace("TELEM ", "TELEM x")),
            Err(DecodeError::BadHex)
        ));
    }

    #[test]
    fn length_errors() {
        // single byte
        assert!(matches!(
            parse_line("TELEM 00"),
            Err(DecodeError::BadLength { .. })
        ));

        // truncated by one byte
        let mut b = good_bytes();
        b.pop();
        assert!(matches!(
            parse_line(&line(&b)),
            Err(DecodeError::BadLength { .. })
        ));

        // one extra byte
        let mut b = good_bytes();
        b.push(0);
        assert!(matches!(
            parse_line(&line(&b)),
            Err(DecodeError::BadLength { .. })
        ));

        // length byte disagrees with actual size
        let mut b = good_bytes();
        b[0] = 0x21;
        assert!(matches!(
            parse_line(&line(&b)),
            Err(DecodeError::BadLength { .. })
        ));
        let mut b = good_bytes();
        b[0] = 0xff;
        assert!(matches!(
            parse_line(&line(&b)),
            Err(DecodeError::BadLength { .. })
        ));

        // internally consistent but wrong size (31-byte packet): 1 + 33 + checksum = 35
        let mut b = vec![0x21u8];
        b.extend(std::iter::repeat(0u8).take(33));
        b.push(checksum_of(&b));
        assert_eq!(b.len(), 35);
        assert!(matches!(
            parse_line(&line(&b)),
            Err(DecodeError::BadLength { .. })
        ));
    }

    #[test]
    fn checksum_detects_every_single_byte_corruption() {
        // Covers packet bytes, RSSI and LQI (indices 1..=34). Index 0 is the length
        // byte (not summed) and 35 is the checksum itself, tested below.
        for i in 1..=34 {
            let mut b = good_bytes();
            b[i] ^= 0x01;
            assert!(
                matches!(parse_line(&line(&b)), Err(DecodeError::BadChecksum { .. })),
                "byte {i}"
            );
        }
        let mut b = good_bytes();
        *b.last_mut().unwrap() ^= 0xff;
        assert!(matches!(
            parse_line(&line(&b)),
            Err(DecodeError::BadChecksum { .. })
        ));
    }

    #[test]
    fn checksum_wraps_at_256() {
        // Maximum possible sum: every byte 0xff
        let pkt = packet(0xffff, 0xffff, 0xff, &[0xff; PAYLOAD_LEN]);
        let t = parse_line(&line(&frame(&pkt, -1, 0xff))).unwrap();
        assert_eq!(t.header.serial, 0xffff);
        assert_eq!(t.header.tick, 0xffff);
        let Payload::Unknown { type_id, raw } = t.payload else {
            panic!()
        };
        assert_eq!(type_id, 0xff);
        assert_eq!(raw, [0xff; PAYLOAD_LEN]);
    }

    #[test]
    fn crc_flag_is_bit_7_only() {
        let pkt = packet(1, 1, 0x7f, &[0; PAYLOAD_LEN]);
        for (lqi, ok) in [
            (0x80u8, true),
            (0xff, true),
            (0x91, true),
            (0x00, false),
            (0x7f, false),
            (0x40, false),
            (0x11, false),
        ] {
            let r = parse_line(&line(&frame(&pkt, -100, lqi)));
            match (ok, r) {
                (true, Ok(t)) => {
                    assert!(t.link.crc_ok);
                    assert_eq!(t.link.lqi, lqi);
                }
                (false, Err(DecodeError::CrcFailed)) => {}
                (_, other) => panic!("lqi {lqi:#04x}: unexpected {other:?}"),
            }
        }
    }

    #[test]
    fn checksum_is_checked_before_crc() {
        // Both wrong: the checksum failure should win.
        let mut b = frame(&packet(1, 1, 0x7f, &[0; PAYLOAD_LEN]), -100, 0x11);
        *b.last_mut().unwrap() ^= 0x01;
        assert!(matches!(
            parse_line(&line(&b)),
            Err(DecodeError::BadChecksum { .. })
        ));
    }

    #[test]
    fn rssi_is_signed_and_scaled() {
        let pkt = packet(1, 1, 0x7f, &[0; PAYLOAD_LEN]);
        for (raw, dbm) in [
            (0i8, -74.0f32),
            (-1, -74.5),
            (63, -42.5),
            (127, -10.5),
            (-48, -98.0),
            (-128, -138.0),
        ] {
            let t = parse_line(&line(&frame(&pkt, raw, 0x91))).unwrap();
            assert_eq!(t.link.rssi_dbm, dbm, "raw {raw}");
        }
    }

    #[test]
    fn header_is_little_endian() {
        let t = parse_line(&line(&frame(
            &packet(0xABCD, 0x1234, 0x7f, &[0; PAYLOAD_LEN]),
            -100,
            0x91,
        )))
        .unwrap();
        assert_eq!(t.header.serial, 0xABCD);
        assert_eq!(t.header.tick, 0x1234);
        assert_eq!(t.header.type_id, 0x7f);

        let t = parse_line(&line(&frame(
            &packet(0, 0, 0x7f, &[0; PAYLOAD_LEN]),
            -100,
            0x91,
        )))
        .unwrap();
        assert_eq!((t.header.serial, t.header.tick), (0, 0));
    }

    // ───────────────────── type dispatch ─────────────────────

    #[test]
    fn every_known_type_dispatches_to_its_variant() {
        let z = [0u8; PAYLOAD_LEN];
        let cases: [(u8, &str); 16] = [
            (0x01, "Sensor"),
            (0x02, "Sensor"),
            (0x03, "Sensor"),
            (0x04, "Config"),
            (0x05, "Location"),
            (0x06, "Satellites"),
            (0x07, "Companion"),
            (0x08, "MegaImu"),
            (0x12, "MegaImu"),
            (0x13, "MegaImu"),
            (0x14, "MegaImu"),
            (0x09, "MegaKalman"),
            (0x15, "MegaKalman"),
            (0x11, "MiniSensor"),
            (0x0A, "MetrumSensor"),
            (0x0B, "MetrumCalibration"),
        ];
        for (id, name) in cases {
            let t = parse_ok(id, &z);
            assert_eq!(variant_name(&t.payload), name, "type {id:#04x}");
        }
    }

    #[test]
    fn unknown_types_are_returned_not_errors() {
        let payload = Pb::default().raw(&[1, 2, 3]).fill(24, 0xAB).build();
        for id in [0x00u8, 0x0C, 0x10, 0x16, 0x7f, 0xff] {
            let t = parse_ok(id, &payload);
            let Payload::Unknown { type_id, raw } = t.payload else {
                panic!("type {id:#04x}")
            };
            assert_eq!(type_id, id);
            assert_eq!(raw, payload);
        }
    }

    #[test]
    fn decoders_reject_foreign_type_ids() {
        let z = [0u8; PAYLOAD_LEN];
        assert!(matches!(
            Sensor::decode(0x04, &z),
            Err(DecodeError::InvalidTypeId)
        ));
        assert!(matches!(
            Sensor::decode(0x00, &z),
            Err(DecodeError::InvalidTypeId)
        ));
        assert!(matches!(
            MegaImu::decode(0x09, &z),
            Err(DecodeError::InvalidTypeId)
        ));
        assert!(matches!(
            MegaImu::decode(0x15, &z),
            Err(DecodeError::InvalidTypeId)
        ));
        assert!(matches!(
            MegaKalman::decode(0x14, &z),
            Err(DecodeError::InvalidTypeId)
        ));
        assert!(matches!(
            MegaKalman::decode(0x08, &z),
            Err(DecodeError::InvalidTypeId)
        ));
    }

    // ───────────────────── Cursor ─────────────────────

    #[test]
    fn cursor_reads_little_endian_and_advances() {
        let data = [0x01, 0x34, 0x12, 0xfe, 0xff, 0x00, 0x00, 0x80, 0xff];
        let mut c = Cursor::new(&data);
        assert_eq!(c.u8().unwrap(), 0x01);
        assert_eq!(c.u16().unwrap(), 0x1234);
        assert_eq!(c.i16().unwrap(), -2);
        assert_eq!(c.i32().unwrap(), i32::from_le_bytes([0, 0, 0x80, 0xff]));
        c.finish();
    }

    #[test]
    fn cursor_signed_edges() {
        let mut c = Cursor::new(&[0xff, 0x80, 0x7f]);
        assert_eq!(c.i8().unwrap(), -1);
        assert_eq!(c.i8().unwrap(), -128);
        assert_eq!(c.i8().unwrap(), 127);

        let mut c = Cursor::new(&[0xff, 0x7f, 0x00, 0x80]);
        assert_eq!(c.i16().unwrap(), i16::MAX);
        assert_eq!(c.i16().unwrap(), i16::MIN);

        let mut c = Cursor::new(&[0xff, 0xff, 0xff, 0x7f, 0x00, 0x00, 0x00, 0x80]);
        assert_eq!(c.i32().unwrap(), i32::MAX);
        assert_eq!(c.i32().unwrap(), i32::MIN);
    }

    #[test]
    fn cursor_truncation() {
        assert!(matches!(Cursor::new(&[]).u8(), Err(DecodeError::Truncated)));
        assert!(matches!(
            Cursor::new(&[1]).u16(),
            Err(DecodeError::Truncated)
        ));
        assert!(matches!(
            Cursor::new(&[1, 2, 3]).i32(),
            Err(DecodeError::Truncated)
        ));
        assert!(matches!(
            Cursor::new(&[1, 2, 3]).bytes::<4>(),
            Err(DecodeError::Truncated)
        ));
        assert!(matches!(
            Cursor::new(&[1, 2]).skip(3),
            Err(DecodeError::Truncated)
        ));

        // exactly at the end is fine; the next read is not
        let mut c = Cursor::new(&[1, 2]);
        assert!(c.u16().is_ok());
        assert!(matches!(c.u8(), Err(DecodeError::Truncated)));
        assert!(Cursor::new(&[1, 2]).skip(2).is_ok());
    }

    #[test]
    fn cursor_failed_read_does_not_advance() {
        let mut c = Cursor::new(&[1, 2, 3]);
        assert!(c.i32().is_err());
        assert_eq!(c.u8().unwrap(), 1); // position unchanged by the failed read
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn cursor_finish_catches_unconsumed_bytes() {
        let mut c = Cursor::new(&[1, 2, 3]);
        c.u8().unwrap();
        c.finish();
    }

    // ───────────────────── GPS ─────────────────────

    #[test]
    fn gps_flags_each_bit_independently() {
        let f = GpsFlags::from_byte(0x00);
        assert!(f.nsats == 0 && !f.valid && !f.running && !f.date_valid && !f.course_valid);

        let f = GpsFlags::from_byte(1 << 4);
        assert!(f.nsats == 0 && f.valid && !f.running && !f.date_valid && !f.course_valid);
        let f = GpsFlags::from_byte(1 << 5);
        assert!(f.nsats == 0 && !f.valid && f.running && !f.date_valid && !f.course_valid);
        let f = GpsFlags::from_byte(1 << 6);
        assert!(f.nsats == 0 && !f.valid && !f.running && f.date_valid && !f.course_valid);
        let f = GpsFlags::from_byte(1 << 7);
        assert!(f.nsats == 0 && !f.valid && !f.running && !f.date_valid && f.course_valid);

        // low nibble is the satellite count and never leaks into the booleans
        let f = GpsFlags::from_byte(0x0f);
        assert!(f.nsats == 15 && !f.valid && !f.running && !f.date_valid && !f.course_valid);
        assert_eq!(GpsFlags::from_byte(0xf0).nsats, 0);
        assert_eq!(GpsFlags::from_byte(0xff).nsats, 15);
    }

    #[test]
    fn gps_mode_mapping() {
        assert!(matches!(GpsMode::from_byte(b'N'), GpsMode::NotValid));
        assert!(matches!(GpsMode::from_byte(b'A'), GpsMode::Autonomous));
        assert!(matches!(GpsMode::from_byte(b'D'), GpsMode::Differential));
        assert!(matches!(GpsMode::from_byte(b'E'), GpsMode::Estimated));
        assert!(matches!(GpsMode::from_byte(b'M'), GpsMode::Manual));
        assert!(matches!(GpsMode::from_byte(b'S'), GpsMode::Simulated));
        assert!(matches!(GpsMode::from_byte(b'a'), GpsMode::Other(0x61))); // case-sensitive
        assert!(matches!(GpsMode::from_byte(0), GpsMode::Other(0)));
        assert!(matches!(GpsMode::from_byte(1), GpsMode::Other(1)));
        assert!(matches!(GpsMode::from_byte(0xff), GpsMode::Other(0xff)));
    }

    #[test]
    fn location_extreme_values() {
        let p = Pb::default()
            .u8(0x0f) // flags: 15 sats, nothing else
            .i16(-1) // altitude below sea level
            .i32(-337_000_000) // southern hemisphere
            .i32(1_511_000_000) // eastern hemisphere
            .u8(99)
            .u8(12)
            .u8(31)
            .u8(23)
            .u8(59)
            .u8(59)
            .u8(255)
            .u8(255)
            .u8(255)
            .u8(b'A')
            .u16(u16::MAX)
            .i16(i16::MIN)
            .u8(179)
            .u8(0xEE) // padding must be ignored
            .build();
        let Payload::Location(l) = parse_ok(0x05, &p).payload else {
            panic!()
        };
        assert_eq!(l.altitude, -1);
        assert_eq!(l.latitude, -337_000_000);
        assert_eq!(l.longitude, 1_511_000_000);
        assert_eq!(
            (l.year, l.month, l.day, l.hour, l.minute, l.second),
            (99, 12, 31, 23, 59, 59)
        );
        assert_eq!((l.pdop, l.hdop, l.vdop), (255, 255, 255));
        assert!(matches!(l.mode, GpsMode::Autonomous));
        assert_eq!(l.ground_speed, 65535); // unsigned
        assert_eq!(l.climb_rate, -32768); // signed
        assert_eq!(l.course, 179);
        assert!(!l.flags.valid);
    }

    #[test]
    fn satellites_first_and_last_entries() {
        let mut pb = Pb::default().u8(12);
        for i in 0..12u8 {
            pb = pb.u8(i + 1).u8(40 + i);
        }
        let p = pb.fill(2, 0xEE).build();
        let Payload::Satellites(s) = parse_ok(0x06, &p).payload else {
            panic!()
        };
        assert_eq!(s.channels, 12);
        assert_eq!((s.sats[0].svid, s.sats[0].c_n_1), (1, 40));
        assert_eq!((s.sats[5].svid, s.sats[5].c_n_1), (6, 45));
        assert_eq!((s.sats[11].svid, s.sats[11].c_n_1), (12, 51)); // would be wrong if the loop misaligned
    }

    #[test]
    fn satellites_zero_and_oversized_channel_counts() {
        // 0 channels: still decodes, entries are whatever was transmitted
        let p = Pb::default().u8(0).fill(26, 0).build();
        let Payload::Satellites(s) = parse_ok(0x06, &p).payload else {
            panic!()
        };
        assert_eq!(s.channels, 0);

        // A corrupt count of 255 does not panic. channels is NOT clamped here;
        // callers must use min(channels, 12) when iterating.
        let p = Pb::default().u8(255).fill(26, 0x11).build();
        let Payload::Satellites(s) = parse_ok(0x06, &p).payload else {
            panic!()
        };
        assert_eq!(s.channels, 255);
    }

    // ───────────────────── Config ─────────────────────

    #[test]
    fn config_unsigned_extremes_and_full_width_strings() {
        let p = Pb::default()
            .u8(0xff)
            .u16(0xffff) // flight: would go negative if parsed as i16
            .u8(255)
            .u8(254)
            .u16(0x8000) // apogee_delay
            .u16(0xffff) // main_deploy
            .u16(0xffff) // flight_log_max
            .raw(b"ABCDEFGH") // callsign with no NUL terminator
            .raw(b"12345678")
            .build();
        let Payload::Config(c) = parse_ok(0x04, &p).payload else {
            panic!()
        };
        assert_eq!(c.dev_type, 0xff);
        assert_eq!(c.flight, 65535);
        assert_eq!((c.config_major, c.config_minor), (255, 254));
        assert_eq!(c.apogee_delay, 32768);
        assert_eq!(c.main_deploy, 65535);
        assert_eq!(c.flight_log_max, 65535);
        assert_eq!(c.callsign, *b"ABCDEFGH");
        assert_eq!(c.version, *b"12345678");
    }

    // ───────────────────── sensor-style payloads ─────────────────────

    #[test]
    fn sensor_v1_fields_and_kinds() {
        let p = Pb::default()
            .u8(3)
            .i16(-2)
            .i16(-3)
            .i16(-4)
            .i16(5)
            .i16(6)
            .i16(7)
            .i16(-8)
            .i16(9)
            .i16(i16::MAX)
            .i16(11)
            .i16(12)
            .i16(13)
            .i16(i16::MIN)
            .build();
        for (id, kind) in [(0x01u8, "metrum"), (0x02, "mini"), (0x03, "nano")] {
            let Payload::Sensor(s) = parse_ok(id, &p).payload else {
                panic!()
            };
            assert_eq!(s.state, 3);
            assert_eq!(s.accel, -2);
            assert_eq!(s.pres, -3);
            assert_eq!(s.sense_m, 7);
            assert_eq!(s.height, i16::MAX);
            assert_eq!(s.accel_minus_g, i16::MIN);
            let ok = match (&s.kind, kind) {
                (SensorKind::TeleMetrumV1, "metrum")
                | (SensorKind::TeleMiniV1, "mini")
                | (SensorKind::TeleNano, "nano") => true,
                _ => false,
            };
            assert!(ok, "type {id:#04x}");
        }
    }

    #[test]
    fn mega_imu_all_kinds_and_signed_fields() {
        let p = Pb::default()
            .u8(255) // orient
            .i16(-1) // accel
            .i32(1_013_250) // pres: needs all 4 bytes (> 65535)
            .i16(-4050) // temp below freezing
            .i16(1)
            .i16(-2)
            .i16(3)
            .i16(4)
            .i16(-5)
            .i16(i16::MIN)
            .i16(7)
            .i16(-8)
            .i16(i16::MAX)
            .build();
        for (id, kind) in [(0x08u8, 0), (0x12, 1), (0x13, 2), (0x14, 3)] {
            let Payload::MegaImu(m) = parse_ok(id, &p).payload else {
                panic!()
            };
            assert_eq!(m.orient, 255);
            assert_eq!(m.accel, -1);
            assert_eq!(m.pres, 1_013_250);
            assert_eq!(m.temp, -4050);
            assert_eq!((m.accel_x, m.accel_y, m.accel_z), (1, -2, 3));
            assert_eq!((m.gyro_x, m.gyro_y, m.gyro_z), (4, -5, i16::MIN));
            assert_eq!((m.mag_x, m.mag_y, m.mag_z), (7, -8, i16::MAX));
            let ok = matches!(
                (&m.kind, kind),
                (ImuKind::Invensense, 0)
                    | (ImuKind::BMX160, 1)
                    | (ImuKind::MPU6000, 2)
                    | (ImuKind::BMI088, 3)
            );
            assert!(ok, "type {id:#04x}");
        }
    }

    #[test]
    fn mega_kalman_fields_and_voltage_ranges() {
        let p = Pb::default()
            .u8(5)
            .i16(4095)
            .i16(2000)
            .raw(&[0, 1, 127, 128, 200, 255]) // sense bytes stay unsigned
            .i32(1_013_250)
            .i16(-1)
            .i16(i16::MAX)
            .i16(i16::MIN)
            .i16(-32)
            .i16(4800)
            .i16(i16::MAX)
            .build();
        for (id, v15) in [(0x09u8, true), (0x15, false)] {
            let Payload::MegaKalman(k) = parse_ok(id, &p).payload else {
                panic!()
            };
            assert_eq!(k.state, 5);
            assert_eq!((k.v_batt, k.v_pyro), (4095, 2000));
            assert_eq!(k.sense, [0, 1, 127, 128, 200, 255]);
            assert_eq!(k.ground_pres, 1_013_250);
            assert_eq!(
                (k.ground_accel, k.accel_plus_g, k.accel_minus_g),
                (-1, i16::MAX, i16::MIN)
            );
            assert_eq!((k.acceleration, k.speed, k.height), (-32, 4800, i16::MAX));
            assert_eq!(matches!(k.voltage, VoltageRange::V15), v15);
            assert_eq!(matches!(k.voltage, VoltageRange::V30), !v15);
        }
    }

    #[test]
    fn metrum_sensor_ignores_trailing_padding() {
        let p = Pb::default()
            .u8(2)
            .i16(-1)
            .i32(1_013_250)
            .i16(2500)
            .i16(-16)
            .i16(160)
            .i16(1234)
            .i16(4100)
            .i16(10)
            .i16(20)
            .fill(6, 0xEE)
            .build();
        let Payload::MetrumSensor(m) = parse_ok(0x0A, &p).payload else {
            panic!()
        };
        assert_eq!(m.state, 2);
        assert_eq!((m.accel, m.pres, m.temp), (-1, 1_013_250, 2500));
        assert_eq!((m.acceleration, m.speed, m.height), (-16, 160, 1234));
        assert_eq!((m.v_batt, m.sense_d, m.sense_m), (4100, 10, 20));
    }

    #[test]
    fn metrum_calibration_skips_leading_and_trailing_padding() {
        let p = Pb::default()
            .fill(3, 0xEE)
            .i32(1_013_250)
            .i16(-5)
            .i16(300)
            .i16(-300)
            .fill(14, 0xEE)
            .build();
        let Payload::MetrumCalibration(m) = parse_ok(0x0B, &p).payload else {
            panic!()
        };
        assert_eq!(m.ground_pres, 1_013_250); // a wrong skip count shows up immediately here
        assert_eq!(
            (m.ground_accel, m.accel_plus_g, m.accel_minus_g),
            (-5, 300, -300)
        );
    }

    #[test]
    fn mini_sensor_ground_pres_is_32_bit() {
        let p = Pb::default()
            .u8(1)
            .i16(4000)
            .i16(11)
            .i16(22)
            .i32(1_013_250)
            .i16(1500)
            .i16(-1)
            .i16(2)
            .i16(3)
            .i32(1_010_000)
            .fill(4, 0xEE)
            .build();
        let Payload::MiniSensor(m) = parse_ok(0x11, &p).payload else {
            panic!()
        };
        assert_eq!(m.pres, 1_013_250);
        assert_eq!(m.temp, 1500);
        assert_eq!((m.acceleration, m.speed, m.height), (-1, 2, 3));
        assert_eq!(m.ground_pres, 1_010_000);
    }

    #[test]
    fn companion_reads_twelve_16_bit_channels() {
        let mut pb = Pb::default().u8(0x0a).u8(100).u8(12);
        for i in 0..11u16 {
            pb = pb.u16(i * 1000);
        }
        let p = pb.u16(0xBEEF).build();
        let Payload::Companion(c) = parse_ok(0x07, &p).payload else {
            panic!()
        };
        assert_eq!((c.board_id, c.update_period, c.channels), (0x0a, 100, 12));
        assert_eq!(c.companion_data[0], 0);
        assert_eq!(c.companion_data[10], 10_000);
        assert_eq!(c.companion_data[11], 0xBEEF); // fails if channels are read as single bytes
    }

    // ───────────────────── whole-capture smoke test ─────────────────────

    #[test]
    #[ignore = "needs the real capture; run with `cargo test -- --ignored --nocapture`"]
    fn whole_capture() {
        let text = std::fs::read_to_string("2026-06-18.telem")
            .expect("put the capture next to Cargo.toml (or fix the path)");
        let (mut ok, mut crc_bad, mut unknown) = (0u32, 0u32, 0u32);
        let mut serials = std::collections::HashSet::new();

        for (n, l) in text
            .lines()
            .enumerate()
            .filter(|(_, l)| !l.trim().is_empty())
        {
            match parse_line(l) {
                Ok(t) => {
                    ok += 1;
                    serials.insert(t.header.serial);
                    if matches!(t.payload, Payload::Unknown { .. }) {
                        unknown += 1;
                    }
                }
                Err(DecodeError::CrcFailed) => crc_bad += 1,
                Err(e) => panic!("line {}: {:?}", n + 1, e),
            }
        }
        println!("ok={ok} crc_failed={crc_bad} unknown_type={unknown} serials={serials:?}");
        assert!(ok > 0);
        assert_eq!(unknown, 0, "a type ID in your capture isn't handled yet");
        assert_eq!(
            serials.len(),
            1,
            "expected a single transmitting device in this capture"
        );
    }
}
