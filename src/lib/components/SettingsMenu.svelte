<script lang="ts">
import { fly } from 'svelte/transition';
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core';
import { page } from '$app/state';

const MIN_BLOCK_SIZE = 5;
const MAX_BLOCK_SIZE = 65535;

let settingsOpen: boolean = $state(false);
let displayPath: string = $state('Loading...');
let blockSize: string = $state('Loading..');
let error: string = $state("");

let { displayedEntriesRefresh } = $props();

async function initSettings() {
    displayPath = await invoke('get_data_path');
    blockSize = await invoke('get_block_size')
}

function toggleSettings() {
    settingsOpen = !settingsOpen;
}

async function changeDataPath() {
    const path = await open({
        multiple: false,
        directory: true
    });

    if (path == null) {
        // User closed the dialog, no need to update anything
        // console.log("Update dialog closed")
        return;
    }

    displayPath = path;
    invoke('update_data_path', {path: path});
    if (page.url.pathname == '/previous-flights') {
        // Show the replays from the new folder path
        displayedEntriesRefresh();
    }
}

async function changeBlockSize(event: FocusEvent) {
    let target = event.currentTarget
    if (target instanceof(HTMLInputElement)) {
        blockSize = target.value;
        let blockNumber = parseInt(blockSize);

        if ((blockNumber < MIN_BLOCK_SIZE) || (blockNumber > MAX_BLOCK_SIZE)) {
            error = `Block size must be between ${MIN_BLOCK_SIZE} and ${MAX_BLOCK_SIZE}`;
            return;
        }

        invoke('update_block_size', {newSize: blockNumber});
        error = "";
    } else {
        error = "Unable to update block size"
    }
}

initSettings();
</script>

<button onclick={toggleSettings} class="open-button">
    Settings
</button>

{#if settingsOpen}
    <aside class="sidebar" transition:fly={{x: 250, duration: 350}}>
        <div class="settings-header"> 
            <h2>Settings</h2>
            <button onclick={toggleSettings} class="close-button">
                Close
            </button>
        </div>

        <div id="error-container">
            <p id="error-message">{error}</p>
        </div>

        <ul class="settings-options">
            <li id="data-folder" class="setting-line">
                <p>Data Path</p>
                <button onclick={changeDataPath}>
                    {displayPath}
                </button>
            </li>
            <li id="line-amount" class="setting-line">
                <p>
                    <span class="has-tooltip" tabindex="-1">
                        CSV Loaded Line Amount
                        <span class="tooltip">
                            The number of lines that will be loaded in memory at a time when replaying a CSV flight. Less powerful systems might need to load less at a time.
                        </span>
                    </span>
                </p>
                <input type="text" value={blockSize} onblur={changeBlockSize}>
            </li>
        </ul>
    </aside>
{/if}

<style>
    button {
        margin-left: auto;
        display: flex;
        align-self: flex-end;
        margin-right: 0.5rem;
    }

    .close-button {
        margin-right: 0.5rem;
        justify-self: flex-start;
        align-self: center;
    }
    
    .sidebar {
    position: fixed;
    right: 0;
    top: 0;
    width: 300px;
    height: 100%;
    background: var(--secondary-color);
    box-shadow: -2px 0 5px rgba(0,0,0,0.1);
    padding: 1rem;
    padding-top: 0.5rem;
    border-color: var(--accent-color);
    }

    .settings-header {
        display: flex;
        align-items: center;
    }

    ul {
        display: flex;
        flex-direction: column;
        justify-content: center;
        list-style-type: none;
        padding: 0;
        flex-wrap: nowrap;
        align-items: flex-start;
    }

    li {
        display: flex;
        flex-direction: row;
        align-items: center;
        white-space: nowrap;
        width: 100%;
        margin-right: 0.5rem;
        padding-right: 0rem;
        align-items: center;
        flex-wrap: nowrap;
    }

    p {
        display: flex;
        padding-left: 0;
        width: fit-content;
        padding-right: 0.5rem;
    }

    li button, input {
        align-self: center;
        display: flex;
        overflow-x: auto;
        overflow-y: hidden;
        scrollbar-color: var(--accent-color);
        scrollbar-width: thin;
        max-width: 75%;
        min-width: 0;          /* lets it shrink inside the flex row */
    }

    li input {
        margin-left: auto;     /* pushes the input to the right, like the button */
    }

    .has-tooltip {
        position: relative;
        text-decoration: underline;
        text-decoration-style: dotted;
        cursor: help;
    }

    .tooltip {
        visibility: hidden;
        opacity: 0;
        position: absolute;
        bottom: 130%;
        left: 0;
        width: max-content;
        max-width: 200px;
        white-space: normal;   /* li is nowrap, so allow the text to wrap here */
        background-color: #333;
        color: #fff;
        padding: 6px 10px;
        border-radius: 4px;
        font-size: 0.85rem;
        text-decoration: none;
        pointer-events: none;
        z-index: 10;
        transition: opacity 0.2s ease-in-out;
    }

    .has-tooltip:hover .tooltip,
    .has-tooltip:focus-visible .tooltip {
        visibility: visible;
        opacity: 1;
    }

    #error-message {
        color: red;
    }

    /* TODO - change look of scrollbar to match the app's theme*/
</style>