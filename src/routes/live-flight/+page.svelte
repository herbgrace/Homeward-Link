<script lang="ts">
import NavigationBar from '$lib/components/NavigationBar.svelte';
import FlightDisplayer from '$lib/components/FlightDisplayer.svelte';

import { onDestroy, onMount } from 'svelte';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface Telemetry {
    accelX: number; accelY: number; accelZ: number;
    rotX: number; rotY: number; rotZ: number;
    temperature: number;
}

interface Status {
    connected: boolean;
    device?: string;
    port?: string;
}

let telemetry = $state<Telemetry | null>(null);
let status = $state<Status>({ connected: false });
let running = $state(false);

// Single source of truth for order, labels, and formatting
const columns: { key: keyof Telemetry; label: string; decimals: number }[] = [
    { key: "accelX",      label: "Acceleration X", decimals: 2 },
    { key: "accelY",      label: "Acceleration Y", decimals: 2 },
    { key: "accelZ",      label: "Acceleration Z", decimals: 2 },
    { key: "rotX",        label: "Rotation X",     decimals: 2 },
    { key: "rotY",        label: "Rotation Y",     decimals: 2 },
    { key: "rotZ",        label: "Rotation Z",     decimals: 2 },
    { key: "temperature", label: "Temperature",    decimals: 2 },
];

const headers: string[] = columns.map((c) => c.label);

const line: string[] = $derived(
telemetry
    ? columns.map((c) => telemetry![c.key].toFixed(c.decimals))
    : columns.map(() => "-")
);


onMount(() => {
    // listen() returns a Promise<UnlistenFn>; keep the promises so cleanup
    // works even if the component is destroyed before they resolve.
    const unlisteners = [
        listen<Telemetry>("telemetry", (e) => (telemetry = e.payload)),
        listen<Status>("serial-status", (e) => (status = e.payload)),
    ];

    return () => {
        unlisteners.forEach((p) => p.then((unlisten) => unlisten()));
        // Optional: stop the backend when leaving this page/component
        invoke("stop_serial");
    };
});

async function startReplay() {
    await invoke("start_serial");
    running = true;
    console.log("Started Listening");
}

async function pauseReplay() {
    await invoke("stop_serial");
    running = false;
    console.log("Stopped Listening");
}

</script>

<main class="container">
<title>Live Flight</title>
<NavigationBar />

<FlightDisplayer 
    isLive={true}
    headers={headers}
    displayLine={line}
    isPlaying={running}
    startReplayMethod={startReplay}
    pauseReplayMethod={pauseReplay}
/> 
</main>