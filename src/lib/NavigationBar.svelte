<script lang="ts">
import { goto } from "$app/navigation"
import { page } from "$app/state";

import SettingsMenu from "./SettingsMenu.svelte";
import '../defaultPalette.css'

let paths: string[] = ['/', '/Previous-Flights', '/Live-Flight', '/Calendar']
let currentRoute: string = $derived(page.url.pathname)
let activeId: number = $derived(paths.findIndex(route => {
    return route.toLowerCase() == currentRoute
}));

let { settingsRefresh } = $props();

function handleClick(index: number) {
    activeId = index;
    let reroute = paths[index].toLowerCase();
    console.log("Rerouting to: " + reroute)
    goto(paths[index].toLowerCase())
}
</script>

<div class="header-bar">
    <div class="nav-bar">
        {#each paths as option, i}
            <button
                class:selected={activeId === i}
                aria-label={option}
                aria-current={currentRoute === option}
                onclick={() => handleClick(i)}
            >
            {#if i == 0}
                Home
            {:else}
                {option.replaceAll("-", " ").replaceAll("/", "")}
            {/if}
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