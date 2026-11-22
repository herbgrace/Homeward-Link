<script lang="ts">
import { page } from "$app/state";
import { onMount, onDestroy } from "svelte";
import { LoaderCircle } from "@lucide/svelte";

import {
    init_manager,
    reset_manager,
    determine_waiting_period,
    get_current_frame_and_increment,
    get_next_frame_and_increment,
} from "$lib/utils/blockManager.svelte"

import FlightDisplayer from "$lib/components/FlightDisplayer.svelte";
import { fileNameOf } from "$lib/utils/savedFlightUtils";
import type { Frame } from "$lib/utils/frames";

let filePath: string = $state(page.url.searchParams.get('filePath') || "");
let isLoading: boolean = $state(true);
let isPlaying: boolean = $state(false);

let frame: Frame | undefined = $state(undefined);

const wait = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

onMount(async () => {
    await init_manager(filePath);
    frame = get_current_frame_and_increment();
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
    while (frame != undefined && isPlaying) {
        await wait(determine_waiting_period())
        frame = get_next_frame_and_increment();
    }
}

async function stop_replay() {
    isPlaying = false;
}
</script>

<svelte:head>
    <title>{fileNameOf(filePath) || "Previous Flight"} · Homeward Link</title>
</svelte:head>

{#if isLoading}
    <div class="loading">
        <span class="spinner"><LoaderCircle size={28} /></span>
        <p>Loading {fileNameOf(filePath)}…</p>
    </div>
{:else}
    <FlightDisplayer
        dataUrl={filePath}
        frame={frame}
        isLive={false}
        isPlaying={isPlaying}
        startReplayMethod={start_replay}
        pauseReplayMethod={stop_replay}
    />
{/if}

<style>
    .loading {
        height: 100%;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 0.75rem;
        color: var(--text-muted);
    }
    .spinner { display: inline-flex; color: var(--accent); animation: spin 0.9s linear infinite; }
</style>
