<script lang="ts">
import { page } from "$app/state";
import { onMount, onDestroy } from "svelte";

import {    
    init_manager, 
    reset_manager, 
    get_current_line, 
    get_headers,
    determine_waiting_period, 
    get_next_line_and_increment
} from "$lib/ts_helpers/blockManager.svelte"

import FlightDisplayer from "$lib/svelte_components/FlightDisplayer.svelte";

let filePath: string = $state(page.url.searchParams.get('filePath') || "");
let isLoading: boolean = $state(true);
let isPlaying: boolean = $state(false);

let headers: string[] = $state([]);
let displayLine: string[] = $state([]);

const wait = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

onMount(async () => {
    await init_manager(filePath);
    headers = get_headers();
    displayLine = get_current_line();
    isLoading = false;
});

onDestroy(async () => {
    isPlaying = false;
    reset_manager();
})

async function start_replay() {
    if (isPlaying) {
        return;
    }
    isPlaying = true;
    while (displayLine.length != 0 && isPlaying) {
        await wait(determine_waiting_period())
        displayLine = get_next_line_and_increment();
    }
}

async function stop_replay() {
    isPlaying = false;
}

</script>


<main class="main-content">
    {#if isLoading}
        Loading...
    {:else}
        <FlightDisplayer 
            dataUrl={filePath}
            headers={headers}
            displayLine={displayLine}
            isLive={false}
            startReplayMethod={start_replay}
            pauseReplayMethod={stop_replay}
        />
    {/if}
</main>