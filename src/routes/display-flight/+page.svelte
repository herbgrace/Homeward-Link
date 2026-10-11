<script lang="ts">
import { page } from "$app/state";
import { goto } from "$app/navigation";
import { invoke } from "@tauri-apps/api/core";

let filePath = $derived(page.url.searchParams.get('filePath'));
let fileContents = $derived(invoke<string[][]>('load_flight_data', {fileName: filePath}))

</script>

<main class="container">
<div class="header-bar">
    <button 
    class="return-button"
    onclick={() => goto("/previous-flights")}
    >
        Return
    </button>

    <h3>{filePath}</h3>
</div>
<div class="flight-content">
    {#await fileContents}
        <p>Loading file contents...</p>
    {:then data} 
        {#each data as line}
            <p>{line}</p>
        {/each}
    {/await}
</div>
</main>

<style>
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
</style>