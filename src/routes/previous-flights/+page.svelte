<script lang="ts">
import { goto } from '$app/navigation';
import NavigationBar from '$lib/svelte_components/NavigationBar.svelte';
import { loadEntries } from '$lib/ts_helpers/savedFlightUtils';

let statusMessage: string = $state('Loading Files...');
let loadedFiles: string[] = $state([]);

async function refreshEntries() {
    // console.log("Hit the refresh entries method")
    let response = await loadEntries();
    loadedFiles = response.data;
    statusMessage = response.errorMessage;
}

function clickEntry(entryIndex: number) {
    if (entryIndex < 0 || entryIndex >= loadedFiles.length) {
        console.log("Tried to load a flight that was out of bound of the saved array")
        statusMessage = "Unable to load saved flight"
        return;
    }
    const queryParams = new URLSearchParams ({
        filePath: loadedFiles[entryIndex]
    });
    goto(`/display-previous-flight?${queryParams.toString()}`)
}

refreshEntries();
</script>

<main class="container">
<title>Previous Flights</title>
<NavigationBar settingsRefresh={refreshEntries}/> 
<p>{statusMessage}</p>

<ul>
    {#each loadedFiles as path, i}
        <li>
            <button 
            id="button-{i}"
            onclick={() => clickEntry(i)}
            >
                {path}
            </button>
        </li>
    {/each}
</ul>
</main>

<style>
    ul {
        display: block;
        padding-left: 0.75rem;
        padding-right: 0.25rem;
        justify-content: center;
    }

    li {
        margin-bottom: 0.5rem;
        list-style: none;
        justify-self: flex-start;
    }
</style>