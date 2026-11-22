<script lang="ts">
    import { Mountain, Gauge, Wind, Compass, RotateCw } from "@lucide/svelte";

    interface Props {
        altitude: number | undefined;   // m, relative to the launch altitude
        speed: number | undefined;      // m/s
        pressure: number | undefined;   // Pa
        tilt: number;                   // degrees from vertical
        roll: number;                   // degrees
        variant?: "tiles" | "hud";
    }

    let { altitude, speed, pressure, tilt, roll, variant = "tiles" }: Props = $props();

    const fmt = (v: number | undefined, digits = 1) =>
        v == undefined || !Number.isFinite(v) ? "—" : v.toFixed(digits);

    const normalizeDeg = (d: number) => ((d % 360) + 540) % 360 - 180;

    let metrics = $derived([
        { label: "Altitude", value: fmt(altitude), unit: "m", icon: Mountain },
        { label: "Speed", value: fmt(speed), unit: "m/s", icon: Gauge },
        { label: "Pressure", value: fmt(pressure == undefined ? undefined : pressure / 100), unit: "hPa", icon: Wind },
        { label: "Tilt", value: fmt(tilt), unit: "°", icon: Compass },
        { label: "Roll", value: fmt(normalizeDeg(roll)), unit: "°", icon: RotateCw },
    ]);
</script>

<div class="metrics {variant}">
    {#each metrics as m}
        <div class="metric">
            <div class="label">
                <m.icon size={variant === "hud" ? 12 : 14} />
                {m.label}
            </div>
            <div class="value tabular">
                {m.value}<span class="unit">{m.value === "—" ? "" : m.unit}</span>
            </div>
        </div>
    {/each}
</div>

<style>
    .metrics.tiles {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
        gap: 0.75rem;
    }

    .tiles .metric {
        position: relative;
        padding: 0.85rem 1rem 0.9rem;
        background: var(--surface);
        border: 1px solid var(--border);
        border-radius: var(--radius);
        box-shadow: var(--shadow-sm);
        overflow: hidden;
    }

    .tiles .metric:first-child::before {
        content: "";
        position: absolute;
        inset: 0 auto 0 0;
        width: 3px;
        background: var(--accent);
    }

    .label {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        font-size: 11.5px;
        font-weight: 600;
        letter-spacing: 0.05em;
        text-transform: uppercase;
        color: var(--text-muted);
    }
    .tiles .label :global(svg) { color: var(--accent); }

    .tiles .value {
        margin-top: 0.3rem;
        font-size: 24px;
        font-weight: 650;
        letter-spacing: -0.02em;
        line-height: 1.1;
    }

    .unit {
        margin-left: 0.25rem;
        font-size: 0.55em;
        font-weight: 550;
        color: var(--text-subtle);
        letter-spacing: 0;
    }

    /* HUD: translucent strip over a 3D view */
    .metrics.hud {
        display: flex;
        gap: 2px;
        padding: 4px;
        border-radius: 10px;
        background: rgba(12, 12, 14, 0.6);
        border: 1px solid rgba(255, 255, 255, 0.1);
        backdrop-filter: blur(10px);
        -webkit-backdrop-filter: blur(10px);
        color: #f2f2f3;
    }

    .hud .metric { padding: 6px 12px; min-width: 84px; }
    .hud .metric + .metric { border-left: 1px solid rgba(255, 255, 255, 0.08); }
    .hud .label { color: rgba(255, 255, 255, 0.6); font-size: 10px; }
    .hud .label :global(svg) { color: #f78f44; }
    .hud .value { font-size: 18px; font-weight: 650; margin-top: 2px; }
    .hud .unit { color: rgba(255, 255, 255, 0.5); }
</style>
