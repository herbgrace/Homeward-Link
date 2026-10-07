<script lang="ts">
import '../defaultPallette.css'

let settingsOpen: boolean = $state(false);
let dataPath: string = $state('');

function toggleSettings() {
    settingsOpen = !settingsOpen;
}

function changeDataPath() {

}
</script>

<button onclick={toggleSettings} class="open-button">
    Settings
</button>

{#if settingsOpen}
    <aside class="sidebar">
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
                    Change Path
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
    background: var(--accent-color);
    box-shadow: -2px 0 5px rgba(0,0,0,0.1);
    padding: 1rem;
    padding-top: 0.5rem;
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
    }

    li button {
        align-self: center;
        display: flex;
    }
</style>