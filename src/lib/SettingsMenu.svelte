<script lang="ts">
import { fly } from 'svelte/transition';
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core';
import '../defaultPalette.css';

let settingsOpen: boolean = $state(false);
let displayPath: string = $state('Loading...');

async function initPath() {
    displayPath = await invoke('get_data_path');
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

    console.log(`Updating file path to ${path}`)
    displayPath = path;
    invoke('update_data_path', {path: path});
}

initPath();
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

    /* TODO - change look of scrollbar to match the app's theme*/
</style>