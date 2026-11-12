<script lang="ts">
    import { onMount } from "svelte";
    import FlightIndicators from "flight-indicators-js"

    let count = 3

    let elements: [HTMLDivElement | null, string][] = Array.from({ length: count }, () => [null, 'attitude']);
    let visualizers: typeof FlightIndicators[] = [];

    let { rollVal = 0, pitchVal = 0 } = $props();

    $effect(() => {
        rollVal;
        pitchVal;

        visualizers.forEach(v => {
            updateHorizon(v);
        })
    })

    onMount(() => {
        elements.forEach(([elem, type]) => {
            if (elem) {
                const instance = new FlightIndicators(elem, type, {
                    size: 180,
                    roll: rollVal,
                    pitch: pitchVal,
                    hideBox: true,
                    imagesDirectory: "/img/visualizations/",
                });
                visualizers.push(instance)
            }
        });
    });

    function updateHorizon(indicator: typeof FlightIndicators) {
        if (indicator) {
            indicator.updateRoll(rollVal);
            indicator.updatePitch(pitchVal);
        }
    }
</script>


{#each elements as [_, type], i}
    <div class="visualizers-stack">
        <div bind:this={elements[i][0]} class="layer"></div>
    </div>
{/each}


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