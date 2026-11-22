<script lang="ts">
    import FlightIndicators from "flight-indicators-js";

    interface Props {
        rollVal: number;
        pitchVal: number;
        vertSpeed: number;
        airspeedVal: number;
        altVal: number;
        pressureVal: number;
        /** grid: 2x2 cards. strip: a single row that fills the available height */
        layout?: "grid" | "strip";
    }

    let {
        rollVal,
        pitchVal,
        vertSpeed,
        airspeedVal,
        altVal,
        pressureVal,
        layout = "grid",
    } : Props = $props();

    const POSSIBLE_TYPES = ["altimeter", "vertical", "airspeed", "attitude"];
    const TYPE_LABELS: Record<string, string> = {
        altimeter: "Altimeter",
        vertical: "Vertical speed",
        airspeed: "Airspeed",
        attitude: "Attitude",
    };

    let current_types = $state([...POSSIBLE_TYPES]);

    type IndicatorRecord = { instance: any; node: HTMLElement };
    let visualizers = new Set<IndicatorRecord>();

    function applyValues({ instance, node }: IndicatorRecord) {
        // Make sure node is still in the DOM and instance exists
        if (!instance || !node || !document.body.contains(node)) return;
        try {
            switch (instance.type) {
                case "attitude":
                    instance.updateRoll(rollVal);
                    instance.updatePitch(pitchVal);
                    break;
                case "altimeter":
                    instance.updateAltitude(altVal);
                    instance.updatePressure(pressureVal);
                    break;
                case "vertical":
                    instance.updateVerticalSpeed(vertSpeed);
                    break;
                case "airspeed":
                    instance.updateAirSpeed(airspeedVal);
                    break;
            }
        } catch (e) {
            console.error("Error updating flight indicator:", e);
        }
    }

    $effect(() => {
        rollVal;
        pitchVal;
        vertSpeed;
        airspeedVal;
        altVal;
        pressureVal;
        visualizers.forEach(applyValues);
    });

    function initIndicator(node: HTMLElement, initialType: string) {
        let currentRecord: IndicatorRecord | null = null;
        let size = 0;

        function fit() {
            const box = node.getBoundingClientRect();
            const next = Math.max(60, Math.floor(Math.min(box.width, box.height)));
            if (currentRecord && next !== size) {
                size = next;
                currentRecord.instance.resize(size);
            }
        }

        function create(type: string) {
            if (currentRecord) {
                visualizers.delete(currentRecord);
                node.innerHTML = "";
            }

            const instance = new FlightIndicators(node, type, {
                size: 120,
                roll: rollVal,
                pitch: pitchVal,
                hideBox: true,
                imagesDirectory: "/img/visualizations",
            });
            instance.type = type;

            currentRecord = { instance, node };
            visualizers.add(currentRecord);
            size = 0;
            fit();
            applyValues(currentRecord);
        }

        create(initialType);

        const observer = new ResizeObserver(fit);
        observer.observe(node);

        return {
            update(newType: string) {
                create(newType);
            },
            destroy() {
                observer.disconnect();
                if (currentRecord) {
                    visualizers.delete(currentRecord);
                    currentRecord = null;
                    node.innerHTML = "";
                }
            }
        };
    }
</script>

<div class="instruments {layout}">
    {#each current_types as type, i}
        <div class="gauge">
            <select
                class="gauge-select"
                aria-label="Instrument {i + 1} type"
                bind:value={current_types[i]}
            >
                {#each POSSIBLE_TYPES as possible_type}
                    <option value={possible_type}>{TYPE_LABELS[possible_type]}</option>
                {/each}
            </select>
            <div class="dial" use:initIndicator={type}></div>
        </div>
    {/each}
</div>

<style>
    .instruments {
        display: grid;
        gap: 0.5rem;
        width: 100%;
        height: 100%;
        min-height: 0;
    }

    .instruments.grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
    .instruments.strip { grid-template-columns: repeat(4, minmax(0, 1fr)); }

    .gauge {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.35rem;
        min-width: 0;
        min-height: 0;
        padding: 0.5rem;
        border-radius: var(--radius-sm);
        background: var(--surface-2);
        border: 1px solid var(--border);
    }

    .grid .gauge { aspect-ratio: 1 / 1.12; }

    .gauge-select {
        appearance: none;
        -webkit-appearance: none;
        border: none;
        background: transparent;
        color: var(--text-muted);
        font: inherit;
        font-size: 11px;
        font-weight: 650;
        letter-spacing: 0.05em;
        text-transform: uppercase;
        text-align: center;
        text-align-last: center;
        padding: 2px 6px;
        border-radius: 4px;
        cursor: pointer;
        max-width: 100%;
    }
    .gauge-select:hover { color: var(--accent-text); background: var(--surface-3); }
    .gauge-select:focus-visible { outline: 2px solid var(--accent); }
    .gauge-select option { text-transform: none; color: var(--text); background: var(--surface); }

    .dial {
        flex: 1;
        width: 100%;
        min-height: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        overflow: hidden;
    }

    /* The library injects .instrument with absolutely-positioned image layers */
    :global(.dial > .instrument) {
        position: relative !important;
        overflow: hidden !important;
        flex-shrink: 0;
    }

    :global(.dial > .instrument .box) {
        width: 100% !important;
        height: 100% !important;
        position: absolute !important;
        top: 0 !important;
        left: 0 !important;
    }

    :global(.dial > .instrument .hidden) { display: none !important; }
</style>
