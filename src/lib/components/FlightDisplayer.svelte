<script lang="ts">
import { goto } from "$app/navigation";
import { onDestroy } from "svelte";
import {
    ArrowLeft, Play, Pause, LayoutList, Clapperboard,
    Square, PanelLeft, LayoutGrid, Radio, Timer,
} from "@lucide/svelte";
import { type Frame, OrientationTracker } from "$lib/utils/frames";
import { toModelPose } from "$lib/utils/modelPose";
import type { CameraMode } from "$lib/utils/camera";
import { fileNameOf, extensionOf } from "$lib/utils/savedFlightUtils";
import CameraView from "./CameraView.svelte";
import FlightVisualizations from "./FlightVisualizations.svelte";
import KeyMetrics from "./KeyMetrics.svelte";

interface Props {
    dataUrl?: string,
    frame: Frame | undefined,
    isLive: boolean,
    isPlaying: boolean,
    startReplayMethod: (event: MouseEvent) => void,
    pauseReplayMethod: (event: MouseEvent) => void,
}

let {
    dataUrl = undefined,
    isLive = false,
    frame,
    isPlaying,
    startReplayMethod,
    pauseReplayMethod
}: Props = $props()

type ViewMode = "info" | "visual";
type VisualLayout = "single" | "focus" | "grid";

const MODE_KEY = "homeward-flight-mode";
const LAYOUT_KEY = "homeward-visual-layout";

function readPref<T extends string>(key: string, allowed: T[], fallback: T): T {
    try {
        const v = localStorage.getItem(key) as T | null;
        if (v && allowed.includes(v)) return v;
    } catch { /* storage unavailable */ }
    return fallback;
}

function writePref(key: string, value: string) {
    try { localStorage.setItem(key, value); } catch { /* storage unavailable */ }
}

let viewMode: ViewMode = $state(readPref(MODE_KEY, ["info", "visual"], "info"));
let visualLayout: VisualLayout = $state(readPref(LAYOUT_KEY, ["single", "focus", "grid"], "focus"));
let infoCamera: CameraMode = $state("chase");
let visualCameras: CameraMode[] = $state(["chase", "ground", "side", "orbit"]);

$effect(() => writePref(MODE_KEY, viewMode));
$effect(() => writePref(LAYOUT_KEY, visualLayout));

let fileName = $derived(fileNameOf(dataUrl));
let fileExt = $derived(extensionOf(dataUrl));

let rocketPath = $state("/models/RussianRocket/RocketTweaked.glb");
let rotation: number[] = $state([0, 0, 0]);
let modelPose = $derived(toModelPose(rotation));
let startingAlt = $state(-1);
let startTimeMs: number | null = $state(null);

// Keep the previous value if the new one is undefined, default to 0 for first values
let speed: number = $derived.by((prev = 0) => {
    if (frame?.metrics?.speedMs) {
        return frame.metrics.speedMs;
    }
    return prev;
});
let currentAlt: number = $derived.by((prev = startingAlt) => {
    if (frame?.metrics?.altitudeM) {
        return frame.metrics.altitudeM - startingAlt;
    }
    return prev;
});
let pressure: number  = $derived.by((prev = 0) => {
    if (frame?.metrics?.pressurePa) {
        return frame.metrics.pressurePa;
    }
    return prev;
});

// Sources without these sensors (e.g. the ESP32 test board) show "-" instead of a fake 0
let hasMetrics = $derived(frame?.metrics != undefined);
let displayAlt = $derived(hasMetrics ? Math.max(currentAlt, 0) : undefined);
let viewerHeight = $derived(Math.max(currentAlt, 0));

let elapsed = $derived(
    frame?.timeMs != null && startTimeMs != null ? Math.max(0, (frame.timeMs - startTimeMs) / 1000) : null
);

function formatElapsed(s: number | null) {
    if (s == null) return "--:--.-";
    const m = Math.floor(s / 60);
    const sec = s - m * 60;
    return `${String(m).padStart(2, "0")}:${sec.toFixed(1).padStart(4, "0")}`;
}

const tracker = new OrientationTracker();

onDestroy(() => {
    tracker.reset();
    rotation = [0, 0, 0];
})

// Frames without orientation data (e.g. GPS packets) keep the last known pose
$effect(() => {
    if (frame) rotation = tracker.update(frame);
    if (startingAlt == -1 && frame?.metrics?.altitudeM != undefined) {
        startingAlt = frame.metrics.altitudeM;
    }
    if (startTimeMs == null && frame?.timeMs != null) {
        startTimeMs = frame.timeMs;
    }
});

function togglePlayback(e: MouseEvent) {
    if (isPlaying) pauseReplayMethod(e);
    else startReplayMethod(e);
}

let visibleViews = $derived(visualLayout === "single" ? 1 : visualLayout === "focus" ? 3 : 4);
</script>

<div class="flight-page" class:visual={viewMode === "visual"}>
    <header class="flight-toolbar">
        <div class="toolbar-left">
            {#if !isLive}
                <button class="btn btn-ghost btn-icon" onclick={() => goto("/previous-flights")} aria-label="Back to previous flights" title="Back to previous flights">
                    <ArrowLeft size={18} />
                </button>
            {/if}
            <div class="flight-title">
                {#if isLive}
                    <span class="live-icon" class:on={isPlaying}><Radio size={16} /></span>
                    <h1>Live Telemetry</h1>
                    <button class="btn btn-secondary reset-orientation-button" onclick={() => rotation = [0, 0, 0]}>Reset Orientation</button>
                    <span class="badge" class:badge-success={isPlaying}>
                        <span class="dot" class:pulse={isPlaying}></span>
                        {isPlaying ? "Receiving" : "Idle"}
                    </span>
                {:else}
                    <h1 title={dataUrl}>{fileName}</h1>
                    {#if fileExt}<span class="badge" class:badge-accent={fileExt === "telem"}>{fileExt}</span>{/if}
                {/if}
            </div>
        </div>

        <div class="toolbar-right">
            <div class="clock tabular" title="Elapsed flight time">
                <Timer size={14} />
                T+ {formatElapsed(elapsed)}
            </div>

            {#if viewMode === "visual"}
                <div class="segmented" role="group" aria-label="Camera layout">
                    <button class:active={visualLayout === "single"} onclick={() => (visualLayout = "single")} title="Single view" aria-label="Single view"><Square size={14} /></button>
                    <button class:active={visualLayout === "focus"} onclick={() => (visualLayout = "focus")} title="Main view + two side views" aria-label="Focus layout"><PanelLeft size={14} /></button>
                    <button class:active={visualLayout === "grid"} onclick={() => (visualLayout = "grid")} title="Four views" aria-label="Grid layout"><LayoutGrid size={14} /></button>
                </div>
            {/if}

            <div class="segmented" role="group" aria-label="Display mode">
                <button class:active={viewMode === "info"} onclick={() => (viewMode = "info")}><LayoutList size={14} />Information</button>
                <button class:active={viewMode === "visual"} onclick={() => (viewMode = "visual")}><Clapperboard size={14} />Visual</button>
            </div>

            <button class="btn btn-primary play-button" onclick={togglePlayback}>
                {#if isPlaying}
                    <Pause size={15} fill="currentColor" />
                    {isLive ? "Stop" : "Pause"}
                {:else}
                    <Play size={15} fill="currentColor" />
                    {isLive ? "Start listening" : "Play"}
                {/if}
            </button>
        </div>
    </header>

    {#snippet hud()}
        <KeyMetrics variant="hud" altitude={displayAlt} speed={hasMetrics ? speed : undefined} pressure={hasMetrics ? pressure : undefined} tilt={rotation[1]} roll={rotation[0]} />
    {/snippet}

    {#if viewMode === "info"}
        <div class="info-layout">
            <aside class="info-side">
                <section class="card viewer-card">
                    <div class="card-header"><span class="card-title">3D View</span></div>
                    <div class="viewer-frame">
                        <CameraView
                            url={rocketPath}
                            rotationX={modelPose.x}
                            rotationY={modelPose.y}
                            rotationZ={modelPose.z}
                            order={modelPose.order}
                            height={viewerHeight}
                            bind:camera={infoCamera}
                            {hud}
                            compact
                        />
                    </div>
                </section>

                <section class="card">
                    <div class="card-header"><span class="card-title">Instruments</span></div>
                    <div class="card-body">
                        <FlightVisualizations
                            rollVal={rotation[0]}
                            pitchVal={rotation[1]}
                            vertSpeed={speed * 3.6}
                            airspeedVal={speed * 3.6}
                            altVal={currentAlt}
                            pressureVal={pressure}
                        />
                    </div>
                </section>
            </aside>

            <main class="info-main">
                <KeyMetrics altitude={displayAlt} speed={hasMetrics ? speed : undefined} pressure={hasMetrics ? pressure : undefined} tilt={rotation[1]} roll={rotation[0]} />

                <section class="card data-card">
                    <div class="card-header">
                        <span class="card-title">Telemetry</span>
                        {#if frame && frame.fields.length > 0}
                            <span class="field-count">{frame.fields.length} fields</span>
                        {/if}
                    </div>
                    {#if !frame || frame.fields.length === 0}
                        <div class="no-data">
                            <p class="no-data-title">No data yet</p>
                            <p>{isLive ? "Start listening to receive telemetry from your device." : "Press Play to start the replay."}</p>
                        </div>
                    {:else}
                        <dl class="fields">
                            {#each frame.fields as f}
                                <div class="cell">
                                    <dt>{f.label}</dt>
                                    <dd class="tabular">{f.value === "" ? "-" : f.value}</dd>
                                </div>
                            {/each}
                        </dl>
                    {/if}
                </section>
            </main>
        </div>
    {:else}
        <div class="visual-layout">
            <div class="views {visualLayout}">
                {#each visualCameras.slice(0, visibleViews) as _, i (i)}
                    <div class="view view-{i}">
                        <CameraView
                            url={rocketPath}
                            rotationX={modelPose.x}
                            rotationY={modelPose.y}
                            rotationZ={modelPose.z}
                            order={modelPose.order}
                            height={viewerHeight}
                            bind:camera={visualCameras[i]}
                            {hud}
                            alwaysShowHud={i === 0}
                            compact={i > 0}
                        />
                    </div>
                {/each}
            </div>

            <div class="instrument-strip card">
                <FlightVisualizations
                    layout="strip"
                    rollVal={rotation[0]}
                    pitchVal={rotation[1]}
                    vertSpeed={speed * 3.6}
                    airspeedVal={speed * 3.6}
                    altVal={currentAlt}
                    pressureVal={pressure}
                />
            </div>
        </div>
    {/if}
</div>

<style>
    .flight-page {
        display: flex;
        flex-direction: column;
        height: 100%;
        min-height: 0;
    }

    /* Toolbar */
    .flight-toolbar {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
        flex-wrap: wrap;
        padding: 0.6rem 1rem;
        background: var(--surface);
        border-bottom: 1px solid var(--border);
        flex-shrink: 0;
    }

    .toolbar-left, .toolbar-right {
        display: flex;
        align-items: center;
        gap: 0.6rem;
        min-width: 0;
    }

    .toolbar-right { flex-wrap: wrap; justify-content: flex-end; }

    .flight-title {
        display: flex;
        align-items: center;
        gap: 0.6rem;
        min-width: 0;
    }

    .flight-title h1 {
        font-size: 15px;
        font-weight: 650;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .live-icon { display: inline-flex; color: var(--text-subtle); }
    .live-icon.on { color: var(--accent); }

    .dot {
        width: 6px;
        height: 6px;
        border-radius: 50%;
        background: currentColor;
    }
    .dot.pulse { animation: pulse 1.4s ease-in-out infinite; }

    .clock {
        display: inline-flex;
        align-items: center;
        gap: 0.4rem;
        height: 32px;
        padding: 0 0.7rem;
        border-radius: var(--radius-sm);
        background: var(--surface-2);
        border: 1px solid var(--border);
        font-size: 13px;
        font-weight: 600;
        color: var(--text);
    }
    .clock :global(svg) { color: var(--accent); }

    .play-button { min-width: 96px; }

    /* Information mode */
    .info-layout {
        flex: 1;
        min-height: 0;
        overflow: auto;
        display: grid;
        grid-template-columns: minmax(300px, 380px) minmax(0, 1fr);
        gap: 1rem;
        padding: 1rem;
        align-items: start;
    }

    .info-side, .info-main {
        display: flex;
        flex-direction: column;
        gap: 1rem;
        min-width: 0;
    }

    .info-side { position: sticky; top: 0; }

    .viewer-card { overflow: hidden; }
    .viewer-frame { height: 300px; padding: 0.6rem; }

    .card-body { padding: 0.75rem; }

    .field-count { font-size: 12px; color: var(--text-subtle); }

    .fields {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
        margin: 0;
    }

    .cell {
        padding: 0.7rem 1rem;
        border-bottom: 1px solid var(--border);
        border-right: 1px solid var(--border);
        min-width: 0;
    }

    dt {
        font-size: 11.5px;
        font-weight: 600;
        color: var(--text-muted);
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    dd {
        margin: 0.15rem 0 0;
        font-size: 15px;
        font-weight: 550;
        color: var(--text);
        overflow-wrap: anywhere;
    }

    .no-data {
        padding: 3rem 1rem;
        text-align: center;
        color: var(--text-muted);
    }
    .no-data-title { font-weight: 600; color: var(--text); margin-bottom: 0.25rem; }

    /* Visual mode */
    .visual-layout {
        flex: 1;
        min-height: 0;
        display: grid;
        grid-template-rows: minmax(0, 1fr) clamp(140px, 22vh, 210px);
        gap: 0.75rem;
        padding: 0.75rem;
    }

    .views {
        display: grid;
        gap: 0.75rem;
        min-height: 0;
    }

    .views.single { grid-template-columns: 1fr; grid-template-rows: 1fr; }

    .views.focus {
        grid-template-columns: minmax(0, 2.2fr) minmax(0, 1fr);
        grid-template-rows: repeat(2, minmax(0, 1fr));
    }
    .views.focus .view-0 { grid-row: 1 / span 2; }

    .views.grid {
        grid-template-columns: repeat(2, minmax(0, 1fr));
        grid-template-rows: repeat(2, minmax(0, 1fr));
    }

    .view { min-height: 0; min-width: 0; }

    .instrument-strip {
        padding: 0.5rem;
        min-height: 0;
    }

    @media (max-width: 900px) {
        .info-layout { grid-template-columns: 1fr; }
        .info-side { position: static; }
        .views.focus { grid-template-columns: 1fr 1fr; grid-template-rows: 2fr 1fr; }
        .views.focus .view-0 { grid-row: auto; grid-column: 1 / span 2; }
    }
</style>
