<script lang="ts" module>
import { goto } from "$app/navigation"

let activeId: number = $state(0);
let pages: string[] = ['Home', 'Previous-Flights', 'Live-Flight', 'Calendar']
let current: string = $state('')

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

<style>
button {
    color:black;
    background: red;
}

button.selected {
    background: blue;
}
</style>