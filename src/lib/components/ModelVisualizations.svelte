<script lang="ts">
    import FlightIndicators from "flight-indicators-js";

    const MAX_COUNT = 6;
    let count = $state(3);
    let { rollVal = 0, pitchVal = 0 } = $props();

    let visualizers = new Set<any>();
    const POSSIBLE_TYPES = ["attitude", "altimeter", "vertical", "coordinator", "heading", "airspeed"]

    let current_types = $state([...POSSIBLE_TYPES]);

    $effect(() => {
        rollVal;
        pitchVal;

        visualizers.forEach((v) => {
            if (v) {
                v.updateRoll(rollVal);
                v.updatePitch(pitchVal);
            }
        });
    });


    function initIndicator(node: HTMLElement, initialType: string) {
    let instance: any;

    function create(type: string) {
        if (instance) {
            visualizers.delete(instance);
            node.innerHTML = "";
        }

        instance = new FlightIndicators(node, type, {
            size: 180,
            roll: rollVal,
            pitch: pitchVal,
            hideBox: true,
            imagesDirectory: "/img/visualizations",
        });

        visualizers.add(instance);
    }

    // Run on initial mount
    create(initialType);

    return {
        // Automatically called by Svelte whenever `type` changes
        update(newType: string) {
            create(newType);
        },
        destroy() {
            if (instance) {
                visualizers.delete(instance);
                node.innerHTML = "";
            }
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
    
    {#each current_types.slice(0, count) as type, i}
        <select 
            name="Visualizer Type" 
            id='visualizer-selection-{i}' 
            bind:value={current_types[i]}
        >
            {#each POSSIBLE_TYPES as possible_type}
                <option 
                    id="type-selection-{i}-{possible_type}" 
                    selected={type==possible_type}
                    value={possible_type}
                >
                    {possible_type}
                </option>
            {/each}
        </select>
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