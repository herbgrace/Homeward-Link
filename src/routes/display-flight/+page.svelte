<script lang="ts">
import { page } from "$app/state";
import { goto } from "$app/navigation";
import { invoke } from "@tauri-apps/api/core";

import RocketRenderer from "./RocketRenderer.svelte";

const COLUMN_COUNT = 2;

let filePath = $derived(page.url.searchParams.get('filePath'));
let fileName = $derived(filePath?.split("\\").at(-1));

let firstBlock: string[][] = $state([]);
let secondBlock: string[][] = $state([]);
let usingFirst: boolean = $state(false); 
let headers: string[] = $state([]);

let reachedEnd: boolean = $state(false);
let displayedLine = $state(0)
let block_size = $derived((usingFirst ? firstBlock.length : secondBlock.length) - 1)
let linesUntilRefresh = $derived(Math.floor(block_size * 0.9))

async function show_next_data_point() {
    // Calculate how long between this data point and the next data point
    // Then show the next data point after that long    
    if (displayedLine == block_size) {
        show_new_block()
        return;
    } else if (displayedLine == block_size - linesUntilRefresh) {
        load_new_block();
    } 

    displayedLine++;
}

function show_new_block() {
    usingFirst = !usingFirst;
    displayedLine = 0;
}

function determine_waiting_period(firstLine: string[], secondLine: string[]): Number {
    // TODO - determine what other headers contain valid time data
    // TODO - find a better way to do this rather than manually listing valid headers
    const DEFAULT_WAIT = 100;
    const SECOND_HEADERS = ["unixtime", "relseconds"];
    const MS_HEADERS = ["relms", "currenttimemillis", "unixms"]

    let usingSeconds: boolean = false;

    let timeIndex = headers.findIndex((head) => {
        if (SECOND_HEADERS.includes(head.toLowerCase())) {
            usingSeconds = true;
            return true;
        } else {
            return MS_HEADERS.includes(head.toLowerCase());
        }
    })

    if (timeIndex == -1) {
        return DEFAULT_WAIT;
    }

    let difference = Number(secondLine[timeIndex]) - Number(firstLine[timeIndex]);
    return usingSeconds ? difference * 1000 : difference;
}

async function load_new_block() {
    // Block will always have headers in the 0th index,
    // The rest of the data will occupy the rest of the space
    let loadedBlock = await invoke<string[][]>('load_flight_block', {filePath: filePath});
    headers = loadedBlock[0];
    // A bit counter-intuitive, but we want to avoid updating the data we're currently displaying
    if (usingFirst) {
        secondBlock = loadedBlock.slice(1);
    } else {
        firstBlock = loadedBlock.slice(1);
    }
}

async function initial_setup() {
    await load_new_block();
    show_new_block();
}

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
        <button onclick={show_next_data_point}>TEMP: show next data point</button>
        <h3>{fileName}</h3>
        <p>TEMP: You have reached the end: {reachedEnd}</p>
        <p>TEMP: Current displayed line: {displayedLine}</p>
    </div>
    <div class="flight-data">
        {#await initial_setup()}
            <p>Loading data...</p>
        {:then d} 
            <table>
                <tbody>
                    {#each Array(Math.ceil(headers.length / COLUMN_COUNT)) as _, row}
                    <tr>
                        {#each Array(COLUMN_COUNT) as _, col}
                        <td>
                            {headers[(row * COLUMN_COUNT) + col]} <br />
                            <!-- TODO - add checking to see if there is any data to display -->
                            {#if usingFirst}
                                {firstBlock[displayedLine][(row * COLUMN_COUNT) + col]}
                            {:else}
                                {secondBlock[displayedLine][(row * COLUMN_COUNT) + col]}
                            {/if}
                        </td>
                        {/each}
                    </tr>
                    {/each}
                </tbody>
            </table>
        {/await}
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