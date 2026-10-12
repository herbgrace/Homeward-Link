<script lang="ts">
import { page } from "$app/state";
import { goto } from "$app/navigation";
import { invoke } from "@tauri-apps/api/core";

import RocketRenderer from "./RocketRenderer.svelte";

let filePath = $derived(page.url.searchParams.get('filePath'));
let fileName = $derived(filePath?.split("\\").at(-1));
let loadedBlock: string[][] = $state([])

async function load_new_block() {
    loadedBlock = await invoke<string[][]>('load_flight_block', {filePath: filePath})
}

load_new_block();

// Front end stores two different blocks at a time
// Seamlessly transitions from 1 -> 2 when 1 ends
// After transitioning from 1 -> 2, requests new block and replaces 1
// Should Tauri state keep track of which block is needed or should front-end?
// Front-end keeping track of block number feels like not its responsibility
// Storing current block in tauri state feels convoluted and confusing, needing resets when going back
// Tauri commands don't have state, so can't track block in function
</script>

<main class="container">

<div class="sidebar">
    <RocketRenderer />
</div>

<div class="main-content">
    <div class="header-bar">
        <button 
        class="return-button"
        onclick={() => goto("/previous-flights")}
        >
            Return
        </button>
        <h3>{fileName}</h3>
    </div>
    <div class="flight-data">
        {#each loadedBlock as line}
            <p>{line}</p>
        {/each}
    </div>
    <button onclick={load_new_block}>Temp Next Block</button>
</div>

</main>

<style>
.container {
    justify-content: flex-end;
    /* max-width: 75%; */
    align-items: flex-end;
    display: flex;
    flex-direction: row;
    margin-right: 0;
}

.header-bar {
    display: flex;
    align-items: center;
    margin-top: auto;
    /* justify-items: flex-start; */
}

button {
    display: flex;
    width: fit-content;
    padding-left: auto;
}

h3 {
    align-self: center;
    display: flex;
    margin-left: 1rem;
}

.sidebar {
    position: fixed;
    left: 0;
    top: 0;
    width: 25%;
    height: 100%;
    background: var(--secondary-color);
    box-shadow: -2px 0 5px rgba(0,0,0,0.1);
    padding: 1rem;
    padding-top: 0.5rem;
    border-color: var(--accent-color);
}

.main-content {
    align-self: right;
    max-width: 70%;
    display: flexbox;
    justify-items: center;
    align-items: center;
}

.flight-data {
    display: flexbox;
    max-width: 100%;
    text-wrap: wrap;
    box-sizing: border-box;
}
</style>