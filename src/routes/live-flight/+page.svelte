<script lang="ts">
import NavigationBar from '$lib/components/NavigationBar.svelte';
import FlightDisplayer from '$lib/components/FlightDisplayer.svelte';

import { onMount } from 'svelte';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { liveToFrame, type Frame } from '$lib/utils/frames';
import type { Snapshot, LiveTelemetry } from "$lib/utils/telemetryDisplay";

let running = $state(false);
let frame: Frame | undefined = $state(undefined);


$effect(() => {
    const unlisten = listen<LiveTelemetry>("telemetry", (e) => {
        frame = liveToFrame(e.payload);
    });
    return () => { unlisten.then((f) => f()); };   // clean up on unmount
});

// onMount(() => {
//     // listen() returns a Promise<UnlistenFn>; keep the promises so cleanup
//     // works even if the component is destroyed before they resolve.
//     const unlisteners = [
//         listen<Snapshot>("flight-snapshot", (e) => { frame = snapshotToFrame(e.payload); }),
//         listen<LiveTelemetry>("telemetry", (e) => { frame = liveESPToFrames(e.payload); }),
//     ];

//     return () => {
//         unlisteners.forEach((p) => p.then((unlisten) => unlisten()));
//         // Optional: stop the backend when leaving this page/component
//         invoke("stop_serial");
//     };
// });

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
    frame={frame}
    isPlaying={running}
    startReplayMethod={startReplay}
    pauseReplayMethod={pauseReplay}
/> 
</main>