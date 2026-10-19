<script lang="ts">
import NavigationBar from '$lib/svelte_components/NavigationBar.svelte';
import FlightDisplayer from '$lib/svelte_components/FlightDisplayer.svelte';

let headers: string[] = $state([])
let line: string[] = $state([]);

function startReplay() {
    console.log("Live flight started... (In theory)")
    // Get the new line from the board
    // Update line based on received information
}

function pauseReplay() {
    console.log("Live flight paused... (In theory)")
    // Stop updating the line variable?
    // Stop reading input from the board?
}

</script>

<main class="container">
<title>Live Flight</title>
<NavigationBar />

<FlightDisplayer 
    isLive={true}
    headers={headers}
    displayLine={line}
    startReplayMethod={startReplay}
    pauseReplayMethod={pauseReplay}
/> 
</main>