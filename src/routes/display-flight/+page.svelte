<script lang="ts">
import { page } from "$app/state";
import { goto } from "$app/navigation";
import { onMount, onDestroy } from "svelte";

import {    
    init_manager, 
    reset_manager, 
    get_current_line, 
    get_headers, 
    get_rotation,
    determine_waiting_period, 
    get_next_line_and_increment
} from "$lib/ts_helpers/blockManager.svelte"
import ModelViewer from "$lib/svelte_components/ModelViewer.svelte";

const COLUMN_COUNT = 2;

let filePath: string = $state(page.url.searchParams.get('filePath') || "");
let fileName: string = $derived(filePath?.split("\\").at(-1) || "");
let isLoading: boolean = $state(true);
let isPlaying: boolean = $state(false);

let headers: string[] = $state([]);
let displayLine: string[] = $state([]);

let rocketPath = $state("/models/RussianRocket/RocketTweaked.glb");
let rocketRoll = $state(0);
let rocketPitch = $state(0);
let rocketYaw = $state(0);

const wait = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

onMount(async () => {
    await init_manager(filePath);
    headers = get_headers();
    displayLine = get_current_line();
    moveModel();
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
        moveModel();
        await wait(determine_waiting_period())
        displayLine = get_next_line_and_increment();
    }
}

async function stop_replay() {
    isPlaying = false;
}

function moveModel() {
    let rotation: number[] = get_rotation();
    console.log(rotation);
    rocketRoll = rotation[0];
    rocketPitch = rotation[1];
    rocketYaw = rotation[2];
}

</script>

<div class="content-container">

<div class="sidebar">
    <ModelViewer 
    url={rocketPath}
    rotationX={rocketRoll}
    rotationY={rocketPitch}
    rotationZ={rocketYaw}
    scale={0.25}
    />
</div>

<main class="main-content">
    <div class="header-bar">
        <button 
        class="return-button"
        onclick={() => goto("/previous-flights")}
        >
            Return
        </button>
        <button onclick={start_replay}>Play</button>
        <button onclick={stop_replay}>Pause</button>
        <h3>{fileName}</h3>
    </div>
    <div class="flight-data">
        {#if isLoading}
            <p>Loading data...</p>
        {:else} 
            <table>
                <tbody>
                    {#each Array(Math.ceil(headers.length / COLUMN_COUNT)) as _, row}
                    <tr>
                        {#each Array(COLUMN_COUNT) as _, col}
                        <td>
                            {headers[(row * COLUMN_COUNT) + col]} <br />

                            <!-- TODO - add checking to see if there is any data to display -->
                            {#if displayLine.length == 0}
                                No Data to display
                            {:else}
                                {displayLine[(row * COLUMN_COUNT) + col]}
                            {/if}
                        </td>
                        {/each}
                    </tr>
                    {/each}
                </tbody>
            </table>
        {/if}
    </div>
</main>

</div>

<style>
/* Global rules are usually best in an external global.css, 
   but kept here to ensure height fills the screen */
:global(html), :global(body) {
    margin: 0;
    padding: 0;
    height: 100%;
}

.content-container {
    display: grid;
    /* Sidebar takes 250px exactly, main content takes 100% of remaining space */
    grid-template-columns: 250px 1fr;
    min-height: 100vh;
    width: 100%;
}

.sidebar {
    background: var(--secondary-color, #f4f4f4);
    box-shadow: 2px 0 5px rgba(0,0,0,0.1); /* Flipped shadow direction to the right */
    padding: 1rem;
    padding-top: 0.5rem;
    border-right: 1px solid var(--accent-color, #ccc);
    box-sizing: border-box;       
    display: flex;                
    flex-direction: column;       
    gap: 1rem;                    
}

.main-content {
    padding: 2rem;
    box-sizing: border-box;
    
    display: flex;
    flex-direction: column;   /* Stack the header-bar and flight-data vertically */
    justify-content: flex-start;  /* Align content vertically */
    align-items: center;      /* Center content horizontally */
    gap: 1.5rem;              /* Add clean spacing between header and flight data */
}

.header-bar {
    display: flex;
    align-items: center;
    gap: 0.5rem;              /* Spacing between buttons and text */
    width: 100%;
    max-width: 800px;         /* Keeps the table and header aligned nicely */
}

.flight-data {
    width: 100%;
    max-width: 800px;         /* Limits content stretch on extremely wide monitors */
    box-sizing: border-box;
}

table {
    width: 100%;
    border-collapse: collapse;
}

td {
    padding: 12px;
    border: none;
}

tr {
    border-bottom: 1px solid #e2e8f0;
}

button {
    padding: 0.5rem 1rem;
    cursor: pointer;
}

h3 {
    margin: 0 0 0 1rem;
}
</style>