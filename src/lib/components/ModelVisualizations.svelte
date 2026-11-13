<script lang="ts">
    import FlightIndicators from "flight-indicators-js";

    interface Props {
        rollVal: number;
        pitchVal: number;
        turnVal: number;
        headingVal: number;
        vertSpeed: number;
        airspeedVal: number;
        altVal: number;
        pressureVal: number;
    }

    let { 
        rollVal,
        pitchVal,
        turnVal,
        headingVal,
        vertSpeed,
        airspeedVal,
        altVal,
        pressureVal,
    } : Props = $props();

    const MAX_COUNT = 6;
    let count = $state(3);

    let visualizers = new Set<{instance: any; node: HTMLElement}>();
    const POSSIBLE_TYPES = ["attitude", "altimeter", "vertical", "coordinator", "heading", "airspeed"]

    let current_types = $state([...POSSIBLE_TYPES]);

    $effect(() => {
        rollVal;
        pitchVal;
        turnVal;
        headingVal;
        vertSpeed;
        airspeedVal;
        altVal;
        pressureVal;

        visualizers.forEach(({ instance, node }) => {
            // Safety check: ensure node is still in the DOM and instance exists
            if (!instance || !node || !document.body.contains(node)) return;

            try {
                switch (instance.type) {
                    case POSSIBLE_TYPES[0]: // Attitude
                        instance.updateRoll(rollVal);
                        instance.updatePitch(pitchVal);
                        break;
                    case POSSIBLE_TYPES[1]: // Altimeter
                        instance.updateAltitude(altVal);
                        instance.updatePressure(pressureVal);
                        break;
                    case POSSIBLE_TYPES[2]: // Vertical
                        instance.updateVerticalSpeed(vertSpeed);
                        break;
                    case POSSIBLE_TYPES[3]: // Coordinator
                        instance.updateCoordinator(turnVal);
                        break;
                    case POSSIBLE_TYPES[4]: // Heading
                        instance.updateHeading(headingVal);
                        break;
                    case POSSIBLE_TYPES[5]: // Airspeed
                        instance.updateAirSpeed(airspeedVal);
                        break;
                    default:
                        break;
                }
            } catch (e) {
                // Catch any stray DOM update errors gracefully
                console.error("Error updating flight indicator:", e);
            }
        });
    });


function initIndicator(node: HTMLElement, initialType: string) {
    let currentRecord: { instance: any; node: HTMLElement } | null = null;

    function create(type: string) {
        if (currentRecord) {
            visualizers.delete(currentRecord);
            node.innerHTML = "";
        }

        const instance = new FlightIndicators(node, type, {
            size: 180,
            roll: rollVal,
            pitch: pitchVal,
            hideBox: true,
            imagesDirectory: "/img/visualizations",
        });

        // Assign type to instance for reference if needed
        instance.type = type;

        currentRecord = { instance, node };
        visualizers.add(currentRecord);
    }

    create(initialType);

    return {
        update(newType: string) {
            create(newType);
        },
        destroy() {
            if (currentRecord) {
                visualizers.delete(currentRecord);
                currentRecord = null;
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