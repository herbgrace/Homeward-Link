<script lang="ts">
import { goto } from "$app/navigation";

import {
    get_rotation
} from "$lib/utils/blockManager.svelte"

import ModelViewer from "$lib/components/ModelViewer.svelte";

interface Props {
    dataUrl?: string,
    displayLine: string[],
    headers: string[],
    isLive: boolean,
    startReplayMethod: (event: MouseEvent) => void,
    pauseReplayMethod: (event: MouseEvent) => void,
}

let {
    dataUrl = undefined,
    isLive = false,
    displayLine,
    headers,
    startReplayMethod,
    pauseReplayMethod
}: Props = $props()

let fileName = $derived(dataUrl?.split("\\").at(-1))

$effect(() => {
    displayLine;
    headers;
    moveModel();
})

const COLUMN_COUNT = 2;

let rocketPath = $state("/models/RussianRocket/RocketTweaked.glb");
let rocketRoll = $state(0);
let rocketPitch = $state(0);
let rocketYaw = $state(0);

function moveModel() {
    let rotation: number[] = get_rotation();
    // console.log(rotation);
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
        <!-- Live Flights have the header bar for navigation -->
        {#if !isLive}
            <button 
            class="return-button"
            onclick={() => goto("/previous-flights")}
            >
                Return
            </button>
            <button onclick={startReplayMethod}>Play</button>
            <button onclick={pauseReplayMethod}>Pause</button>
            <h3>{fileName}</h3>
        {:else}
            <!-- For now, I'm sure logic of getting connected to a board will be different -->
            <button onclick={startReplayMethod}>Play</button>
            <button onclick={pauseReplayMethod}>Pause</button>
        {/if}
    </div>
    <div class="flight-data">
        {#if displayLine.length == 0 || headers.length == 0}
            <p>Invalid data</p>
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