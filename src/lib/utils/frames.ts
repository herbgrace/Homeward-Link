import type { Snapshot, RocketRotation } from "./telemetryDisplay";
import { invoke } from "@tauri-apps/api/core";

const adc = (raw: number) => (raw / 4095) * 3.3;
const pyroV = (raw: number, r: "V15" | "V30") => adc(raw) * (r === "V30" ? 112 / 12 : 127 / 27);
const sense12 = (b: number) => (b << 4) | (b >> 4);
const baroAlt = (pa: number) => 44330.77 * (1 - Math.pow(pa / 101325, 0.190284));
// 3-5 confirmed by your CSV; the rest are from memory
const STATES = ["startup", "idle", "pad", "boost", "fast", "coast", "drogue", "main", "landed", "invalid"];

export interface Field { label: string; value: string }

export interface Frame {
    fields: Field[];                 // what the grid displays, in order
    timeMs: number | null;           // for replay timing
    rotation: RocketRotation | null; // null = this frame has no orientation data
}

export interface LiveTelemetry {
    accelX: number; accelY: number; accelZ: number;
    rotX: number; rotY: number; rotZ: number;
    temperature: number;
}

export interface Status {
    connected: boolean;
    device?: string;
    port?: string;
}

// Single source of truth for order, labels, and formatting
const live_columns: { key: keyof LiveTelemetry; label: string; decimals: number }[] = [
    { key: "accelX",      label: "Acceleration X", decimals: 2 },
    { key: "accelY",      label: "Acceleration Y", decimals: 2 },
    { key: "accelZ",      label: "Acceleration Z", decimals: 2 },
    { key: "rotX",        label: "Rotation X",     decimals: 2 },
    { key: "rotY",        label: "Rotation Y",     decimals: 2 },
    { key: "rotZ",        label: "Rotation Z",     decimals: 2 },
    { key: "temperature", label: "Temperature",    decimals: 2 },
];

const field = (label: string, value: string | number): Field => ({ label, value: String(value) });

// ───────────── CSV ─────────────

const SECOND_HEADERS = ["unixtime", "relseconds", "time"];
const MS_HEADERS = ["relms", "currenttimemillis", "unixms"];
const ROLL_HEADERS = ["roll", "x"];
const PITCH_HEADERS = ["pitch", "y"];
const YAW_HEADERS = ["yaw", "z"];

/** `headers` is the block's first row; `rows` is everything after it. */
export function csvToFrames(headers: string[], rows: string[][]): Frame[] {
    const lower = headers.map((h) => h.toLowerCase());
    const find = (names: string[]) => lower.findIndex((h) => names.includes(h));

    const msIdx = find(MS_HEADERS);
    const secIdx = find(SECOND_HEADERS);
    const timeIdx = msIdx !== -1 ? msIdx : secIdx;
    const timeScale = msIdx !== -1 ? 1 : 1000;

    const rotIdx = [find(ROLL_HEADERS), find(PITCH_HEADERS), find(YAW_HEADERS)];
    const hasRotation = rotIdx.some((i) => i !== -1);

    return rows.map((row) => {
        const t = timeIdx === -1 ? NaN : Number(row[timeIdx]);
        return {
            fields: headers.map((h, i) => field(h, row[i] ?? "")),
            timeMs: Number.isFinite(t) ? t * timeScale : null,
            rotation: hasRotation
                ? (rotIdx.map((i) => (i === -1 ? 0 : Number(row[i]) || 0)) as RocketRotation)
                : null,
        };
    });
}

export function liveTelemetryToFrames(data: LiveTelemetry): Frame | undefined {
    let fields: Field[] = [];

    for (const key of Object.keys(data)) {
        let col = live_columns.find(c => c.key == key);
        let value = data[key as keyof typeof data];
        fields.push({
            label: col ? col.label : key, // Prioritize nicely formatted label, but default to given if not found
            value: value.toFixed(col ? col.decimals : 2)
        })
    }

    return {
        fields: fields,
        timeMs: 0,
        rotation: null
    }
}

export function snapshotToFrame(s: Snapshot): Frame {
    const { imu, kalman: k, gps, sats, config } = s.latest;
    const fields: Field[] = [];
    const add = (label: string, v: string | number | null | undefined) =>
        fields.push({ label, value: v == null ? "—" : String(v) });

    add("Time", `${s.timeS.toFixed(2)} s`);
    add("State", k && (STATES[k.state] ?? k.state));

    add("Height", k && `${k.height} m`);
    add("Speed", k && `${(k.speed / 16).toFixed(1)} m/s`);
    add("Acceleration", k && `${(k.acceleration / 16).toFixed(1)} m/s²`);

    add("Pressure", imu && `${imu.pres} Pa`);
    add("Baro altitude", imu && `${baroAlt(imu.pres).toFixed(1)} m`);
    add("Temperature", imu && `${(imu.temp / 100).toFixed(1)} °C`);

    add("Tilt", imu && `${imu.orient}°`);
    add("Accel (raw)", imu && `${imu.accelX} / ${imu.accelY} / ${imu.accelZ}`);
    add("Gyro (raw)", imu && `${imu.gyroX} / ${imu.gyroY} / ${imu.gyroZ}`);

    add("Mag", imu && [imu.magX, imu.magY, imu.magZ].map((v) => (v / 4096).toFixed(2)).join(" / "));
    add("Battery", k && `${(adc(k.vBatt) * 1.56).toFixed(2)} V`);   // 1.56 fitted from your CSV
    add("Pyro battery", k && `${pyroV(k.vPyro, k.voltage).toFixed(2)} V`);

    add("Drogue / Main", k && `${pyroV(sense12(k.sense[4]), k.voltage).toFixed(2)} / ${pyroV(sense12(k.sense[5]), k.voltage).toFixed(2)} V`);

    add("GPS", gps && `${(gps.latitude / 1e7).toFixed(6)}, ${(gps.longitude / 1e7).toFixed(6)}`);
    add("GPS altitude", gps && `${gps.altitude} m`);
    add("Satellites used / seen", gps && sats && `${gps.flags.nsats} / ${sats.channels}`);

    add("Ground speed", gps && `${(gps.groundSpeed / 100).toFixed(1)} m/s`);
    add("Callsign", config && String.fromCharCode(...config.callsign.filter((c) => c !== 0)));
    add("RSSI / LQI", `${s.link.rssiDbm.toFixed(0)} dBm / ${s.link.lqi & 0x7f}`);

    return { fields, timeMs: s.timeS * 1000, rotation: null };
}