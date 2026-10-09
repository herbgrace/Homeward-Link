<script lang="ts">
import { goto } from "$app/navigation"
import SettingsMenu from "./SettingsMenu.svelte";
import '../defaultPalette.css'

let activeId: number = $state(0);
let pages: string[] = ['Home', 'Previous-Flights', 'Live-Flight', 'Calendar']
let current: string = $state('')

let { settingsRefresh } = $props();

function handleClick(index: number) {
    activeId = index;
    current = pages[index]

    if (current === "Home") {
        goto ('/')
    } else {
        goto(`/${current.toLowerCase()}`)
    }
}
</script>

<div class="header-bar">
    <div class="nav-bar">
        {#each pages as option, i}
            <button
                class:selected={activeId === i}
                aria-label={option}
                aria-current={current === option}
                onclick={() => handleClick(i)}
            >
                {option.replaceAll("-", " ")}
            </button>
        {/each}
    </div>

    <SettingsMenu displayedEntriesRefresh={settingsRefresh}/>
</div>

<style>
.header-bar {
    margin-left: 0.5rem;
    display: flex;
    align-items: flex-end;
}

.nav-bar {
    display: flex;
    align-items: flex-start;
}

button {
    color:var(--button-text);
    background: var(--button-background);
    padding-top: 0.25rem;
    padding-bottom: 0.25rem;
    padding-right: 0.5rem;
    padding-left: 0.5rem;
    margin: 0.5rem;
}

button.selected {
    background: var(--button-active-background);
}
</style>