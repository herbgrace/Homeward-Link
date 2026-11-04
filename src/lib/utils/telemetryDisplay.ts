export type RocketRotation = [roll: number, pitch: number, yaw: number];

export interface LinkInfo { rssiDbm: number; lqi: number; crcOk: boolean }

export interface GpsFlags { nsats: number; valid: boolean; running: boolean; dateValid: boolean; courseValid: boolean }

export interface GpsLocation {            // not "Location": that collides with the DOM type
    flags: GpsFlags; altitude: number; latitude: number; longitude: number;
    year: number; month: number; day: number; hour: number; minute: number; second: number;
    pdop: number; hdop: number; vdop: number; mode: unknown;
    groundSpeed: number; climbRate: number; course: number;
}

export interface MegaImu {
    orient: number; accel: number; pres: number; temp: number;
    accelX: number; accelY: number; accelZ: number;
    gyroX: number; gyroY: number; gyroZ: number;
    magX: number; magY: number; magZ: number; kind: string;
}

export interface MegaKalman {
    state: number; vBatt: number; vPyro: number; sense: number[];
    groundPres: number; groundAccel: number; accelPlusG: number; accelMinusG: number;
    acceleration: number; speed: number; height: number; voltage: "V15" | "V30";
}

export interface Satellites { channels: number; sats: { svid: number; cN1: number }[] }

export interface Config {
    devType: number; flight: number; configMajor: number; configMinor: number;
    apogeeDelay: number; mainDeploy: number; flightLogMax: number; callsign: number[]; version: number[];
}

export interface FlightState {            // Rust None arrives as null
    imu: MegaImu | null; kalman: MegaKalman | null; gps: GpsLocation | null;
    sats: Satellites | null; config: Config | null;
}

export interface Snapshot { uptimeMs: number; timeS: number; link: LinkInfo; latest: FlightState }

export interface LoadStats { lines: number; ok: number; crcFailed: number; otherErrors: number }

export type FlightData =
    | { kind: "table"; data: string[][] }
    | { kind: "telemetry"; data: { snapshots: Snapshot[]; stats: LoadStats } };

export interface EspData {
    accelX: number; accelY: number; accelZ: number;
    rotX: number; rotY: number; rotZ: number;
    temperature: number;
}

// Single source of truth for order, labels, and formatting
export const live_columns: { key: keyof EspData; label: string; decimals: number }[] = [
    { key: "accelX",      label: "Acceleration X", decimals: 2 },
    { key: "accelY",      label: "Acceleration Y", decimals: 2 },
    { key: "accelZ",      label: "Acceleration Z", decimals: 2 },
    { key: "rotX",        label: "Rotation X",     decimals: 2 },
    { key: "rotY",        label: "Rotation Y",     decimals: 2 },
    { key: "rotZ",        label: "Rotation Z",     decimals: 2 },
    { key: "temperature", label: "Temperature",    decimals: 2 },
];

export type LiveTelemetry =
    | { kind: "esp"; data: EspData }
    | { kind: "teleDongle"; data: Snapshot };