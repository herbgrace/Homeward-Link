<script lang="ts">
import { page } from "$app/state";
import { Rocket, House, Radio, FolderClock, Settings } from "@lucide/svelte";

import SettingsMenu from "./SettingsMenu.svelte";

const links = [
    { href: "/", label: "Home", icon: House, match: ["/"] },
    { href: "/live-flight", label: "Live Flight", icon: Radio, match: ["/live-flight"] },
    { href: "/previous-flights", label: "Previous Flights", icon: FolderClock, match: ["/previous-flights", "/display-previous-flight"] },
];

let currentRoute = $derived(page.url.pathname.toLowerCase());
let settingsOpen = $state(false);
</script>

<header class="header-bar">
    <a href="/" class="brand" aria-label="Homeward Link home">
        <span class="brand-mark"><Rocket size={16} strokeWidth={2.25} /></span>
        <span class="brand-name">Homeward Link</span>
    </a>

    <nav class="nav-links">
        {#each links as link}
            {@const active = link.match.includes(currentRoute)}
            <a href={link.href} class="nav-link" class:active aria-current={active ? "page" : undefined}>
                <link.icon size={16} />
                <span>{link.label}</span>
            </a>
        {/each}
    </nav>

    <button class="btn btn-ghost btn-icon settings-button" aria-label="Settings" title="Settings" onclick={() => (settingsOpen = true)}>
        <Settings size={18} />
    </button>
</header>

<SettingsMenu bind:open={settingsOpen} />

<style>
.header-bar {
    height: var(--header-height);
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 1.5rem;
    padding: 0 1rem 0 1.25rem;
    background: var(--surface);
    border-bottom: 1px solid var(--border);
    position: relative;
    z-index: 20;
}

.brand {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    color: var(--text);
    text-decoration: none;
    flex-shrink: 0;
}

.brand-mark {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 8px;
    background: var(--accent);
    color: var(--on-accent);
    box-shadow: 0 2px 8px var(--accent-ring);
}

.brand-name {
    font-weight: 700;
    font-size: 15px;
    letter-spacing: -0.01em;
}

.nav-links {
    display: flex;
    align-items: stretch;
    height: 100%;
    gap: 0.25rem;
}

.nav-link {
    position: relative;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0 0.75rem;
    color: var(--text-muted);
    font-weight: 550;
    font-size: 13.5px;
    text-decoration: none;
    transition: color 0.15s;
}

.nav-link:hover { color: var(--text); }

.nav-link.active { color: var(--text); }

.nav-link.active::after {
    content: "";
    position: absolute;
    left: 0.6rem;
    right: 0.6rem;
    bottom: -1px;
    height: 2px;
    border-radius: 2px 2px 0 0;
    background: var(--accent);
}

.nav-link.active :global(svg) { color: var(--accent); }

.settings-button { margin-left: auto; }

@media (max-width: 720px) {
    .brand-name { display: none; }
    .nav-link span { display: none; }
    .header-bar { gap: 0.5rem; }
}
</style>
