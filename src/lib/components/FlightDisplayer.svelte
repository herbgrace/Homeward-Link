<script lang="ts">
import { goto } from "$app/navigation";
import ModelViewer from "$lib/components/ModelViewer.svelte";
import { type Frame, OrientationTracker, FindField } from "$lib/utils/frames";
import { onDestroy } from "svelte";
import { toModelPose } from "$lib/utils/modelPose";
import ModelVisualizations from "./ModelVisualizations.svelte";

interface Props {
    dataUrl?: string,
    frame: Frame | undefined,
    isLive: boolean,
    isPlaying: boolean,
    startReplayMethod: (event: MouseEvent) => void,
    pauseReplayMethod: (event: MouseEvent) => void,
}

let {
    dataUrl = undefined,
    isLive = false,
    frame,
    isPlaying,
    startReplayMethod,
    pauseReplayMethod
}: Props = $props()

let fileName = $derived(dataUrl?.split(/[\\/]/).at(-1))   // works with / and \ paths

let rocketPath = $state("/models/RussianRocket/RocketTweaked.glb");
let rotation: number[] = $state([0, 0, 0]);
let modelPose = $derived(toModelPose(rotation));

let speed: number = $derived(
    Number.parseFloat(
        FindField(frame, 'speed')?.slice(0, -3) ?? '0'
    ) // In m/s
)
let altitude: number = $derived(
    Number.parseFloat(
        FindField(frame, 'height')?.slice(0, -1) ?? '0'
    )
)
let pressure: number = $derived(
    Number.parseFloat(
        FindField(frame, 'pressure')?.slice(0, -2) ?? '0'
    ) * 0.01 // From Pa to hPa
)

const tracker = new OrientationTracker();

onDestroy(() => {
    tracker.reset();
    rotation = [0, 0, 0];
})

// Frames without orientation data (e.g. GPS packets) keep the last known pose
$effect(() => {
    if (frame) rotation = tracker.update(frame);
});
</script>

<div class="content-container">
    <div class="sidebar">
        <ModelViewer
            url={rocketPath}
            rotationX={modelPose.x}
            rotationY={modelPose.y}
            rotationZ={modelPose.z}
            order={modelPose.order}
            scale={0.25}
        />
        
        <!-- TODO - Determine which axis is correct for roll/pitch -->
        <ModelVisualizations 
            rollVal={modelPose.x}
            pitchVal={modelPose.y}
            turnVal={modelPose.z} // Sets rotation of the TC
            headingVal={0} // Heading angle in degrees for an heading indicator
            vertSpeed={speed * 0.06} // Vertical speed in km/min for the vertical speed indicator
            airspeedVal={speed * 60} // Air speed in meters/hour for an air speed indicator
            altVal={altitude} // Altitude in meters for an altimeter indicator
            pressureVal={pressure} // Pressure in hPa for an altimeter indicator
        />
    </div>

    <main class="main-content">
        <div class="header-bar">
            {#if !isLive}
                <button class="return-button" onclick={() => goto("/previous-flights")}>Return</button>
            {/if}

            <button onclick={(e) => (isPlaying ? pauseReplayMethod(e) : startReplayMethod(e))}>
                {#if isPlaying}
                    Pause
                {:else}
                    Play
                {/if}
            </button>

            {#if !isLive}
                <h3>{fileName}</h3>
            {/if}
        </div>

        <div class="flight-data">
            {#if !frame || frame.fields.length === 0}
                <p>No data to display</p>
            {:else}
                <dl class="fields">
                    {#each frame.fields as f}
                        <div class="cell">
                            <dt>{f.label}</dt>
                            <dd>{f.value}</dd>
                        </div>
                    {/each}
                </dl>
            {/if}
        </div>
    </main>
</div>

<style>
    :global(html), :global(body) {
        margin: 0;
        padding: 0;
        height: 100%;
    }

    .content-container {
        display: grid;
        grid-template-columns: 250px 1fr;
        min-height: 100vh;
        width: 100%;
    }

    .sidebar {
        background: var(--secondary-color, #f4f4f4);
        box-shadow: 2px 0 5px rgba(0,0,0,0.1);
        padding: 1rem;
        padding-top: 0.5rem;
        border-right: 1px solid var(--accent-color, #ccc);
        box-sizing: border-box;       
        display: flex;                            
        flex-direction: column;       
        gap: 1rem;                    
        width: 100%;
        overflow-x: hidden; /* Prevents any child elements from breaking out horizontally */
    }

    .sidebar > :global(*) {
        width: 100%;
        max-width: 100%;
        box-sizing: border-box;
    }

    .main-content {
        padding: 2rem;
        box-sizing: border-box;
        display: flex;
        flex-direction: column;
        justify-content: flex-start;
        align-items: center;
        gap: 1.5rem;
    }

    .header-bar {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        width: 100%;
        max-width: 800px;
    }

    .flight-data {
        width: 100%;
        max-width: 800px;
        box-sizing: border-box;
    }

    button {
        padding: 0.5rem 1rem;
        cursor: pointer;
    }

    h3 {
        margin: 0 0 0 1rem;
    }

    .fields {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        margin: 0;
    }

    .cell {
        padding: 12px;
        border-bottom: 1px solid #e2e8f0;
    }

    dt { font-weight: 600; }
    dd { margin: 0; }
</style>