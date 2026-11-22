<script lang="ts">
import { fly, fade } from 'svelte/transition';
import { invoke } from '@tauri-apps/api/core';
import { X, Folder, Monitor, Sun, Moon } from '@lucide/svelte';
import { chooseDataFolder, getDataPath } from '$lib/utils/savedFlightUtils';
import { theme, setTheme, type ThemePreference } from '$lib/utils/theme.svelte';

const MIN_BLOCK_SIZE = 5;
const MAX_BLOCK_SIZE = 65535;

let { open = $bindable(false) } = $props();

let displayPath: string = $state('Loading...');
let blockSize: string = $state('');
let error: string = $state("");
let saved: boolean = $state(false);

const themeOptions: { value: ThemePreference; label: string; icon: typeof Sun }[] = [
    { value: "system", label: "System", icon: Monitor },
    { value: "light", label: "Light", icon: Sun },
    { value: "dark", label: "Dark", icon: Moon },
];

async function initSettings() {
    try {
        displayPath = await getDataPath();
        blockSize = String(await invoke('get_block_size'));
    } catch (e) {
        error = "Unable to load settings";
    }
}

async function changeDataPath() {
    const path = await chooseDataFolder();
    if (path != null) displayPath = path;
}

async function changeBlockSize() {
    const blockNumber = parseInt(blockSize);
    if (Number.isNaN(blockNumber) || blockNumber < MIN_BLOCK_SIZE || blockNumber > MAX_BLOCK_SIZE) {
        error = `Block size must be between ${MIN_BLOCK_SIZE} and ${MAX_BLOCK_SIZE}`;
        saved = false;
        return;
    }
    await invoke('update_block_size', { newSize: blockNumber });
    error = "";
    saved = true;
    setTimeout(() => (saved = false), 1500);
}

function onKeydown(e: KeyboardEvent) {
    if (open && e.key === "Escape") open = false;
}

$effect(() => {
    if (open) initSettings();
});
</script>

<svelte:window onkeydown={onKeydown} />

{#if open}
    <div class="backdrop" transition:fade={{ duration: 150 }} onclick={() => (open = false)} role="presentation"></div>

    <aside class="panel" transition:fly={{ x: 360, duration: 250, opacity: 1 }} aria-label="Settings">
        <header class="panel-header">
            <h2>Settings</h2>
            <button class="btn btn-ghost btn-icon" aria-label="Close settings" onclick={() => (open = false)}>
                <X size={18} />
            </button>
        </header>

        <div class="panel-body">
            <section class="group">
                <h3 class="group-title">Appearance</h3>
                <div class="setting stacked">
                    <div class="setting-label">
                        <span>Theme</span>
                        <small>Follow the system or pick a mode.</small>
                    </div>
                    <div class="segmented">
                        {#each themeOptions as opt}
                            <button class:active={theme.preference === opt.value} onclick={() => setTheme(opt.value)}>
                                <opt.icon size={14} />
                                {opt.label}
                            </button>
                        {/each}
                    </div>
                </div>
            </section>

            <section class="group">
                <h3 class="group-title">Data</h3>
                <div class="setting stacked">
                    <div class="setting-label">
                        <span>Flight log folder</span>
                        <small>Where Homeward Link looks for .telem and .csv files.</small>
                    </div>
                    <div class="path-row">
                        <div class="path" title={displayPath}><Folder size={15} /><span>{displayPath}</span></div>
                        <button class="btn" onclick={changeDataPath}>Change</button>
                    </div>
                </div>

                <div class="setting stacked">
                    <div class="setting-label">
                        <label for="block-size">CSV lines loaded at a time</label>
                        <small>How many lines are held in memory while replaying a CSV flight. Lower this on less powerful machines.</small>
                    </div>
                    <div class="path-row">
                        <input
                            id="block-size"
                            class="input tabular"
                            type="number"
                            min={MIN_BLOCK_SIZE}
                            max={MAX_BLOCK_SIZE}
                            bind:value={blockSize}
                            onblur={changeBlockSize}
                            onkeydown={(e) => e.key === "Enter" && changeBlockSize()}
                        />
                        {#if saved}<span class="saved" transition:fade={{ duration: 120 }}>Saved</span>{/if}
                    </div>
                </div>

                {#if error}
                    <p class="error" role="alert">{error}</p>
                {/if}
            </section>
        </div>
    </aside>
{/if}

<style>
    .backdrop {
        position: fixed;
        inset: 0;
        background: var(--overlay);
        z-index: 90;
    }

    .panel {
        position: fixed;
        top: 0;
        right: 0;
        bottom: 0;
        width: min(380px, 100vw);
        background: var(--surface);
        border-left: 1px solid var(--border);
        box-shadow: var(--shadow-lg);
        z-index: 100;
        display: flex;
        flex-direction: column;
    }

    .panel-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        height: var(--header-height);
        padding: 0 0.75rem 0 1.25rem;
        border-bottom: 1px solid var(--border);
    }

    .panel-header h2 { font-size: 16px; }

    .panel-body {
        flex: 1;
        overflow-y: auto;
        padding: 0.5rem 1.25rem 1.5rem;
    }

    .group { padding: 1rem 0; border-bottom: 1px solid var(--border); }
    .group:last-child { border-bottom: none; }

    .group-title {
        font-size: 11px;
        font-weight: 650;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        color: var(--text-subtle);
        margin-bottom: 0.75rem;
    }

    .setting {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
        padding: 0.5rem 0;
    }

    .setting.stacked { flex-direction: column; align-items: stretch; gap: 0.6rem; }
    .setting.stacked .segmented { align-self: flex-start; }

    .setting-label { display: flex; flex-direction: column; gap: 0.15rem; }
    .setting-label span, .setting-label label { font-weight: 550; }
    .setting-label small { color: var(--text-muted); font-size: 12px; line-height: 1.45; }

    .path-row { display: flex; align-items: center; gap: 0.5rem; }

    .path {
        flex: 1;
        min-width: 0;
        display: flex;
        align-items: center;
        gap: 0.5rem;
        height: 34px;
        padding: 0 0.7rem;
        border-radius: var(--radius-sm);
        background: var(--surface-2);
        border: 1px solid var(--border);
        color: var(--text-muted);
        font-size: 12.5px;
    }
    .path :global(svg) { flex-shrink: 0; color: var(--accent); }
    .path span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

    #block-size { width: 120px; }

    .saved { color: var(--success); font-size: 12px; font-weight: 550; }

    .error {
        margin-top: 0.5rem;
        padding: 0.55rem 0.75rem;
        border-radius: var(--radius-sm);
        background: var(--danger-soft);
        color: var(--danger);
        font-size: 12.5px;
    }
</style>
