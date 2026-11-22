<script lang="ts">
import FlightDisplayer from '$lib/components/FlightDisplayer.svelte';

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { liveToFrame, type Frame } from '$lib/utils/frames';
import type { LiveTelemetry } from "$lib/utils/telemetryDisplay";

let running = $state(false);
let frame: Frame | undefined = $state(undefined);


$effect(() => {
    const unlisten = listen<LiveTelemetry>("telemetry", (e) => {
        frame = liveToFrame(e.payload);
    });
    return () => { unlisten.then((f) => f()); };   // clean up on unmount
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

<svelte:head>
    <title>Live Flight · Homeward Link</title>
</svelte:head>

<FlightDisplayer
    isLive={true}
    frame={frame}
    isPlaying={running}
    startReplayMethod={startReplay}
    pauseReplayMethod={pauseReplay}
/>
