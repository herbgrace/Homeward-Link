<script lang="ts">
import { page } from "$app/state";
import { goto } from "$app/navigation";
import { onMount } from "svelte";

import { init_manager, get_current_line, get_current_line_and_increment, 
    get_headers, determine_waiting_period, get_next_line_and_increment} from "./blockManager.svelte"
import RocketRenderer from "./RocketRenderer.svelte";

const COLUMN_COUNT = 2;

let filePath: string = $state(page.url.searchParams.get('filePath') || "");
let fileName: string = $derived(filePath?.split("\\").at(-1) || "");
let isLoading: boolean = $state(true);

let headers: string[] = $state([]);
let displayLine: string[] = $state([]);

const wait = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

onMount(async () => {
    await init_manager(filePath);
    headers = get_headers();
    displayLine = get_current_line();
    isLoading = false;
    console.log("Headers: " + headers)
    console.log("Line: " + displayLine)
});

async function start_replay() {
    while (displayLine.length != 0) {
        await wait(determine_waiting_period())
        displayLine = get_next_line_and_increment();
    }
}

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
        <button onclick={start_replay}>Start the show</button>
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