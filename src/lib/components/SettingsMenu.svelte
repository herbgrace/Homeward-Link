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
        console.log("Update dialog closed")
        return;
    }

    // console.log(`Updating file path to ${path}`)
    displayPath = path;
    invoke('update_data_path', {path: path});
    // console.log(page.url.pathname)
    if (page.url.pathname == '/previous-flights') {
        // Show the logs from the new folder path
        // console.log("Attempting to refresh entries")
        displayedEntriesRefresh();
    }
}

async function changeBlockSize(event: FocusEvent) {
    let target = event.currentTarget
    if (target instanceof(HTMLInputElement)) {
        blockSize = target.value;
        let blockNumber = parseInt(blockSize);

        if ((blockNumber < MIN_BLOCK_SIZE) || (blockNumber > MAX_BLOCK_SIZE)) {
            console.log("New block size outside of accepted bounds");
            return;
        }
        // console.log(`New block size: ${blockSize}`)
        invoke('update_block_size', {newSize: blockNumber});
        // console.log("Should have updated the block size")
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

        <ul class="settings-options">
            <li class="data-folder">
                <p>Data Path</p>
                <button onclick={changeDataPath}>
                    {displayPath}
                </button>
            </li>
            <li>
                <div class="line-amount">
                    Loaded Line Amount
                    <span class="block-description">
                        This is a description of what this setting does
                    </span>
                    <input type="text" value={blockSize} onblur={changeBlockSize}>
                    <!-- TODO - show error message if new size is outside of bounds -->
                </div>
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
    width: 250px;
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
    }

    p {
        display: flex;
        padding-left: 0;
        width: fit-content;
        padding-right: 0.5rem;
    }

    li button {
        align-self: center;
        display: flex;
        overflow-x: auto;
        overflow-y: hidden;
        scrollbar-color: var(--accent-color);
        scrollbar-width: thin;
    }

    .line-amount {
        position: relative;
        display: inline-block;
        text-decoration: underline;
        text-decoration-style: dotted;
    }

    .block-description {
        visibility: hidden;
        position: absolute;
        bottom: 120%; /* Position it right above the button */
        left: 50%;
        transform: translateX(-50%);
        background-color: #333;
        color: #fff;
        padding: 6px 10px;
        border-radius: 4px;
        white-space: nowrap;
        opacity: 0;
        transition: opacity 0.2s ease-in-out; /* Smooth fade-in */
    }

    .line-amount:hover .block-description {
        display: inline;
        visibility: visible;
    }

    /* TODO - make the tooltip for hovering over number of loaded lines appear */
    /* TODO - change look of scrollbar to match the app's theme*/
</style>