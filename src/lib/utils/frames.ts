import type { Snapshot, RocketMotion, EspData, LiveTelemetry} from "./telemetryDisplay";
import { live_columns } from "./telemetryDisplay";

const adc = (raw: number) => (raw / 4095) * 3.3;
const pyroV = (raw: number, r: "V15" | "V30") => adc(raw) * (r === "V30" ? 112 / 12 : 127 / 27);
const sense12 = (b: number) => (b << 4) | (b >> 4);
const baroAlt = (pa: number) => 44330.77 * (1 - Math.pow(pa / 101325, 0.190284));

const STATES = ["startup", "idle", "pad", "boost", "fast", "coast", "drogue", "main", "landed", "invalid"];
const MAX_DT_S = 0.5;   // don't integrate a stale rate across a gap of lost packets
const ESP_GYRO_IN_RAD = true;   // Adafruit's example prints rad/s
const GYRO_DPS_PER_LSB = 2000 / 32768;   // from the board's ±2000°/s spec, not checked on your unit

export interface Field { label: string; value: string }

export interface Frame {
    fields: Field[];                 // what the grid displays, in order
    timeMs: number | null;           // for replay timing
    rotation: RocketMotion | null; // null = this frame has no orientation data
}

export function FindField(frame: Frame | undefined, field: string): string | undefined {
    return frame?.fields.find(
        (f) => f.label.toLowerCase() === field.toLowerCase()
    )?.value;
}

export class OrientationTracker {
    private lastMs: number | null = null;
    private lastRate = [0, 0, 0];   // °/s, held until the next sample
    private pose = [0, 0, 0];

    reset() { this.lastMs = null; this.lastRate = [0, 0, 0]; this.pose = [0, 0, 0]; }

    update(f: Frame): number[] {
        const r = f.rotation;
        if (r) {
            if (r.rate && f.timeMs != null) {
                if (this.lastMs != null) {
                    const dt = Math.min(Math.max((f.timeMs - this.lastMs) / 1000, 0), MAX_DT_S);
                    const axes = r.tilt != null ? 1 : 3;   // tilt reference => roll only
                    for (let i = 0; i < axes; i++) {
                        this.pose[i] = (this.pose[i] + this.lastRate[i] * dt) % 360;
                    }
                }
                this.lastMs = f.timeMs;
                this.lastRate = r.rate;
            }
            if (r.tilt != null) this.pose[1] = r.tilt;
        }
        return [...this.pose];
    }
}

export interface Status {
    connected: boolean;
    device?: string;
    port?: string;
}

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

    const gyroIdx = ["gyro_roll", "gyro_pitch", "gyro_yaw"].map((h) => lower.indexOf(h));
    const legacyIdx = [find(ROLL_HEADERS), find(PITCH_HEADERS), find(YAW_HEADERS)];
    const rateIdx = gyroIdx.every((i) => i !== -1) ? gyroIdx
                : legacyIdx.some((i) => i !== -1) ? legacyIdx : null;
    const tiltIdx = lower.indexOf("tilt");

    return rows.map((row) => {
        const num = (i: number) => (i === -1 ? 0 : Number(row[i]) || 0);
        const motion: RocketMotion | null = rateIdx || tiltIdx !== -1
            ? { rate: rateIdx ? (rateIdx.map(num)) : undefined,
                tilt: tiltIdx !== -1 ? num(tiltIdx) : undefined } as RocketMotion
            : null;
        const t = timeIdx === -1 ? NaN : Number(row[timeIdx]);

        return {
            fields: headers.map((h, i) => field(h, row[i] ?? "")),
            timeMs: Number.isFinite(t) ? t * timeScale : null,
            rotation: motion
        };
    });
}

export function liveToFrame(t: LiveTelemetry): Frame {
    switch (t.kind) {
        case "esp":        return liveESPToFrames(t.data);
        case "teleDongle": return snapshotToFrame(t.data);   // same renderer as file replay
    }
}

export function liveESPToFrames(d: EspData): Frame {
    const k = ESP_GYRO_IN_RAD ? 180 / Math.PI : 1;
    return {
        fields: live_columns.map((c) => field(c.label, d[c.key].toFixed(c.decimals))),
        timeMs: performance.now(),    // no device clock: stamp arrival time
        rotation: { rate: [-d.rotY * k, -d.rotX * k, d.rotZ * k] } as RocketMotion,
        // Hardcoded for my ESP32 mounting... make variable?
    };
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
    add("Battery", k && `${(adc(k.vBatt) * 1.56).toFixed(2)} V`);
    add("Pyro battery", k && `${pyroV(k.vPyro, k.voltage).toFixed(2)} V`);

    add("Drogue / Main", k && `${pyroV(sense12(k.sense[4]), k.voltage).toFixed(2)} / ${pyroV(sense12(k.sense[5]), k.voltage).toFixed(2)} V`);

    add("GPS", gps && `${(gps.latitude / 1e7).toFixed(6)}, ${(gps.longitude / 1e7).toFixed(6)}`);
    add("GPS altitude", gps && `${gps.altitude} m`);
    add("Satellites used / seen", gps && sats && `${gps.flags.nsats} / ${sats.channels}`);

    add("Ground speed", gps && `${(gps.groundSpeed / 100).toFixed(1)} m/s`);
    add("Callsign", config && String.fromCharCode(...config.callsign.filter((c) => c !== 0)));
    add("RSSI / LQI", `${s.link.rssiDbm.toFixed(0)} dBm / ${s.link.lqi & 0x7f}`);

    return {
        fields,
        timeMs: s.timeS * 1000,
        rotation: imu ? {
            rate: [imu.gyroX, imu.gyroY, imu.gyroZ].map((v) => v * GYRO_DPS_PER_LSB),
            tilt: imu.orient,
        } as RocketMotion : null,
    };
}