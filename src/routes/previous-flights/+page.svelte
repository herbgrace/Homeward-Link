<script lang="ts">
import NavigationBar from '../../lib/NavigationBar.svelte';
import { loadEntries } from './savedFlightUtils';

let statusMessage: string = $state('Loading Files...');
let loadedFiles: string[] = $state([]);

async function refreshEntries() {
    // console.log("Hit the refresh entries method")
    let response = await loadEntries();
    loadedFiles = response.data;
    statusMessage = response.errorMessage;
}

refreshEntries();
</script>

<main class="container">
<title>Previous Flights</title>
<NavigationBar settingsRefresh={refreshEntries}/> 
<p>This is the Previous Flights</p>

<p>{statusMessage}</p>

<ul>
    {#each loadedFiles as log}
        <li>{log}</li>
    {/each}
</ul>
</main>

<style>
    ul {
        display: block;
        padding-left: 0.25rem;
        padding-right: 0.25rem;
        justify-content: center;
    }

    li {
        margin-bottom: 0.5rem;
        list-style: none;
        justify-self: flex-start;
    }
</style>