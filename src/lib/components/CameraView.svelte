<script lang="ts">
    import type { Snippet } from "svelte";
    import { Maximize2, Minimize2, Video } from "@lucide/svelte";
    import ModelViewer from "./ModelViewer.svelte";
    import { CAMERA_OPTIONS, type CameraMode } from "$lib/utils/camera";

    interface Props {
        url: string;
        rotationX: number;
        rotationY: number;
        rotationZ: number;
        order: "XYZ" | "XZY" | "YXZ" | "YZX" | "ZXY" | "ZYX";
        height: number;
        scale?: number;
        camera?: CameraMode;
        /** Overlay drawn on top of the view (e.g. live readouts) */
        hud?: Snippet;
        /** Show the HUD even when not fullscreen */
        alwaysShowHud?: boolean;
        compact?: boolean;
    }

    let {
        url, rotationX, rotationY, rotationZ, order, height,
        scale = 0.25,
        camera = $bindable("chase"),
        hud,
        alwaysShowHud = false,
        compact = false,
    }: Props = $props();

    let root: HTMLDivElement;
    let nativeFullscreen = $state(false);
    let overlayFullscreen = $state(false);   // fallback when the Fullscreen API isn't available
    let expanded = $derived(nativeFullscreen || overlayFullscreen);

    async function toggleFullscreen() {
        if (expanded) {
            if (document.fullscreenElement) await document.exitFullscreen().catch(() => {});
            overlayFullscreen = false;
            return;
        }
        try {
            await root.requestFullscreen();
        } catch {
            overlayFullscreen = true;
        }
    }

    function onFullscreenChange() {
        nativeFullscreen = document.fullscreenElement === root;
    }

    function onKeydown(e: KeyboardEvent) {
        if (overlayFullscreen && e.key === "Escape") overlayFullscreen = false;
    }
</script>

<svelte:document onfullscreenchange={onFullscreenChange} />
<svelte:window onkeydown={onKeydown} />

<div class="camera-view" class:overlay={overlayFullscreen} class:expanded class:compact bind:this={root}>
    <ModelViewer {url} {rotationX} {rotationY} {rotationZ} {order} {height} {scale} {camera} />

    <div class="controls">
        <label class="camera-select" title={CAMERA_OPTIONS.find((c) => c.value === camera)?.hint}>
            <Video size={13} />
            <select bind:value={camera} aria-label="Camera angle">
                {#each CAMERA_OPTIONS as opt}
                    <option value={opt.value}>{opt.label}</option>
                {/each}
            </select>
        </label>
        <button
            class="fs-button"
            onclick={toggleFullscreen}
            aria-label={expanded ? "Exit fullscreen" : "Fullscreen"}
            title={expanded ? "Exit fullscreen (Esc)" : "Fullscreen"}
        >
            {#if expanded}<Minimize2 size={14} />{:else}<Maximize2 size={14} />{/if}
        </button>
    </div>

    {#if hud && (alwaysShowHud || expanded)}
        <div class="hud">{@render hud()}</div>
    {/if}
</div>

<style>
    .camera-view {
        position: relative;
        width: 100%;
        height: 100%;
        min-height: 0;
        border-radius: var(--radius);
        overflow: hidden;
        background: var(--viewport-bg);
        border: 1px solid var(--border);
        isolation: isolate;
    }

    .camera-view.expanded { border-radius: 0; border: none; }

    .camera-view.overlay {
        position: fixed;
        inset: 0;
        z-index: 200;
        width: 100vw;
        height: 100vh;
    }

    .controls {
        position: absolute;
        top: 10px;
        right: 10px;
        display: flex;
        gap: 6px;
        z-index: 2;
        opacity: 0.85;
        transition: opacity 0.15s;
    }
    .camera-view:hover .controls, .controls:focus-within { opacity: 1; }

    .camera-select, .fs-button {
        display: flex;
        align-items: center;
        gap: 6px;
        height: 28px;
        border-radius: 7px;
        background: rgba(14, 14, 16, 0.62);
        border: 1px solid rgba(255, 255, 255, 0.12);
        color: #f2f2f3;
        backdrop-filter: blur(8px);
        -webkit-backdrop-filter: blur(8px);
    }

    .camera-select { padding: 0 4px 0 9px; position: relative; }
    .camera-select :global(svg) { color: #f78f44; flex-shrink: 0; }

    .camera-select select {
        appearance: none;
        -webkit-appearance: none;
        border: none;
        background: transparent;
        color: inherit;
        font: inherit;
        font-size: 12px;
        font-weight: 600;
        padding: 0 18px 0 0;
        height: 100%;
        cursor: pointer;
        background-image: linear-gradient(45deg, transparent 50%, #c9c9cc 50%), linear-gradient(135deg, #c9c9cc 50%, transparent 50%);
        background-position: calc(100% - 9px) 50%, calc(100% - 5px) 50%;
        background-size: 4px 4px;
        background-repeat: no-repeat;
    }
    .camera-select select:focus { outline: none; }
    .camera-select select option { color: #111; background: #fff; }

    .fs-button {
        width: 28px;
        justify-content: center;
        padding: 0;
    }
    .fs-button:hover, .camera-select:hover { background: rgba(14, 14, 16, 0.8); border-color: rgba(247, 143, 68, 0.6); }

    .compact .controls { top: 8px; right: 8px; }

    .hud {
        position: absolute;
        left: 12px;
        bottom: 12px;
        z-index: 2;
        pointer-events: none;
    }
</style>
