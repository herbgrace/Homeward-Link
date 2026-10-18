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

<main class="container">

<div class="sidebar">
    <ModelViewer 
    url={rocketPath}
    rotationX={rocketRoll}
    rotationY={rocketPitch}
    rotationZ={rocketYaw}
    scale={0.25}
    />
</div>

<div class="main-content">
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
    box-sizing: border-box;       /* Keeps padding inside the 100% height/width */
    display: flex;                /* Turns it into a flex container */
    flex-direction: column;       /* Stacks children vertically */
    gap: 1rem;                    /* Adds spacing between sidebar items */
}

.main-content {
    /* align-self: right; */
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