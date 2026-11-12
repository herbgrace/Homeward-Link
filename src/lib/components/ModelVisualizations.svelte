<script lang="ts">
    import FlightIndicators from "flight-indicators-js";
    import { fill } from "three/src/extras/TextureUtils.js";

    const MAX_COUNT = 5;
    let count = $state(3);
    let { rollVal = 0, pitchVal = 0 } = $props();

    let visualizers = new Set<any>();
    const initTypes = ["attitude", "altimeter", "vertical"];
    const filledTypes = Array(MAX_COUNT - initTypes.length).fill('attitude');
    const indicatorTypes = [...initTypes, ...filledTypes];

    $effect(() => {
        rollVal;
        pitchVal;

        visualizers.forEach((v) => {
            v.updateRoll(rollVal);
            v.updatePitch(pitchVal);
        });
    });

    function initIndicator(node: HTMLElement, type: string) {
        const instance = new FlightIndicators(node, type, {
            size: 180,
            roll: rollVal,
            pitch: pitchVal,
            hideBox: true,
            imagesDirectory: "/img/visualizations",
        });

        visualizers.add(instance);

        return {
            destroy() {
                // Clean up reference when count decreases and element is removed
                visualizers.delete(instance);
            }
        };
    }
</script>

<main>
    <input 
        type="number" 
        id="visualizer-count" 
        min="0" 
        max={MAX_COUNT} 
        bind:value={count}
    >
    
    {#each indicatorTypes.slice(0, count) as type}
        <div class="visualizers-stack">
            <div use:initIndicator={type} class="layer"></div>
        </div>
    {/each}
</main>

<style>
    .visualizers-stack {
        display: grid;
        width: 100%;
        justify-items: center;
        position: relative;
    }

    .layer {
        grid-area: 1 / 1;
        width: 100%;
        display: flex;
        justify-content: center;
    }

    /* Force the library's injected .instrument and its internal image boxes to respect the 180px boundary */
    :global(.layer > .instrument) {
        width: 180px !important;
        height: 180px !important;
        position: relative !important;
        overflow: hidden !important;
    }

    :global(.layer > .instrument img.box),
    :global(.layer > .instrument .box) {
        width: 100% !important;
        height: 100% !important;
        position: absolute !important;
        top: 0 !important;
        left: 0 !important;
    }
</style>