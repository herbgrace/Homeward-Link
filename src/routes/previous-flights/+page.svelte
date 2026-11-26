<script lang="ts">
import { goto } from '$app/navigation';
import { untrack } from 'svelte';
import {
    Search, RefreshCw, FolderOpen, Folder, FileSpreadsheet, FileText,
    ChevronRight, ArrowDownWideNarrow, TriangleAlert, Inbox, SearchX,
} from '@lucide/svelte';
import {
    loadEntryDetails, getDataPath, chooseDataFolder, formatBytes, type SavedFlight,
} from '$lib/utils/savedFlightUtils';
import { libraryEvents } from '$lib/utils/libraryEvents.svelte';
import { openPath } from '@tauri-apps/plugin-opener';

type SortKey = "newest" | "oldest" | "name" | "size";
type TypeFilter = "all" | "telem" | "csv";

let flights: SavedFlight[] = $state([]);
let dataPath = $state("");
let isLoading = $state(true);
let errorMessage = $state("");

let query = $state("");
let sortKey: SortKey = $state("newest");
let typeFilter: TypeFilter = $state("all");

const dateFormat = new Intl.DateTimeFormat(undefined, { month: "short", day: "numeric", year: "numeric" });
const timeFormat = new Intl.DateTimeFormat(undefined, { hour: "numeric", minute: "2-digit" });

let counts = $derived({
    all: flights.length,
    telem: flights.filter((f) => f.extension === "telem").length,
    csv: flights.filter((f) => f.extension === "csv").length,
});

let visible = $derived.by(() => {
    const q = query.trim().toLowerCase();
    const list = flights.filter((f) =>
        (typeFilter === "all" || f.extension === typeFilter) &&
        (q === "" || f.name.toLowerCase().includes(q))
    );
    const byTime = (f: SavedFlight) => f.modifiedMs ?? 0;
    switch (sortKey) {
        case "newest": return list.sort((a, b) => byTime(b) - byTime(a));
        case "oldest": return list.sort((a, b) => byTime(a) - byTime(b));
        case "name":   return list.sort((a, b) => a.name.localeCompare(b.name, undefined, { numeric: true }));
        case "size":   return list.sort((a, b) => b.sizeBytes - a.sizeBytes);
    }
});

async function refreshEntries() {
    isLoading = true;
    try { dataPath = await getDataPath(); } catch { dataPath = ""; }
    const response = await loadEntryDetails();
    flights = response.data;
    errorMessage = response.ok ? "" : response.errorMessage;
    isLoading = false;
}

async function changeFolder() {
    await chooseDataFolder();   // bumps libraryEvents, which triggers a refresh below
}

async function openSavedFolder() {
    if (dataPath && dataPath != "") {
        // console.log("Attempting to open " + dataPath);
        await openPath(dataPath);
    }
}

function openFlight(flight: SavedFlight) {
    const queryParams = new URLSearchParams({ filePath: flight.path });
    goto(`/display-previous-flight?${queryParams.toString()}`);
}

function baseName(name: string) {
    const dot = name.lastIndexOf(".");
    return dot > 0 ? name.slice(0, dot) : name;
}

// Initial load, and reload whenever the data folder changes (here or in Settings)
$effect(() => {
    libraryEvents.version;
    untrack(refreshEntries);
});
</script>

<svelte:head>
    <title>Previous Flights · Homeward Link</title>
</svelte:head>

<div class="page">
    <header class="page-header">
        <div class="title-block">
            <h1>Previous Flights</h1>
            <p class="subtitle">
                {#if isLoading}
                    Loading flight logs…
                {:else}
                    {flights.length} {flights.length === 1 ? "flight log" : "flight logs"}
                    {#if dataPath}
                        in 
                        <button onclick={openSavedFolder} class="btn btn-ghost" id="open-folder-btn">
                            <span class="folder" title={dataPath}><Folder size={13} />{dataPath}</span>
                        </button>
                    {/if}
                {/if}
            </p>
        </div>
        <div class="header-actions">
            <button class="btn btn-icon" title="Refresh" aria-label="Refresh list" onclick={refreshEntries} disabled={isLoading}>
                <span class:spinning={isLoading}><RefreshCw size={15} /></span>
            </button>
            <button class="btn" onclick={changeFolder}>
                <FolderOpen size={15} />
                Change folder
            </button>
        </div>
    </header>

    <div class="toolbar">
        <label class="search">
            <Search size={15} />
            <input class="input" type="search" placeholder="Search flights" bind:value={query} aria-label="Search flights" />
        </label>

        <div class="segmented" role="group" aria-label="Filter by file type">
            <button class:active={typeFilter === "all"} onclick={() => (typeFilter = "all")}>
                All <span class="count">{counts.all}</span>
            </button>
            <button class:active={typeFilter === "telem"} onclick={() => (typeFilter = "telem")}>
                Telemetry <span class="count">{counts.telem}</span>
            </button>
            <button class:active={typeFilter === "csv"} onclick={() => (typeFilter = "csv")}>
                CSV <span class="count">{counts.csv}</span>
            </button>
        </div>

        <label class="sort">
            <ArrowDownWideNarrow size={15} />
            <select class="select" bind:value={sortKey} aria-label="Sort flights">
                <option value="newest">Newest first</option>
                <option value="oldest">Oldest first</option>
                <option value="name">Name (A-Z)</option>
                <option value="size">Largest first</option>
            </select>
        </label>
    </div>

    <section class="card list-card">
        <div class="list-head" aria-hidden="true">
            <span>Name</span>
            <span>Type</span>
            <span class="col-date">Modified</span>
            <span class="col-size">Size</span>
            <span></span>
        </div>

        {#if isLoading}
            {#each Array(5) as _}
                <div class="row skeleton-row">
                    <div class="name-cell"><span class="sk sk-icon"></span><span class="sk sk-text"></span></div>
                    <span class="sk sk-badge"></span>
                    <span class="sk sk-short col-date"></span>
                    <span class="sk sk-tiny col-size"></span>
                    <span></span>
                </div>
            {/each}
        {:else if errorMessage}
            <div class="state">
                <div class="state-icon danger"><TriangleAlert size={22} /></div>
                <h2>Couldn't load your flights</h2>
                <p>{errorMessage}</p>
                <div class="state-actions">
                    <button class="btn" onclick={refreshEntries}><RefreshCw size={15} />Try again</button>
                    <button class="btn btn-primary" onclick={changeFolder}><FolderOpen size={15} />Choose folder</button>
                </div>
            </div>
        {:else if flights.length === 0}
            <div class="state">
                <div class="state-icon"><Inbox size={22} /></div>
                <h2>No flight logs yet</h2>
                <p>Homeward Link looks for <code>.telem</code> and <code>.csv</code> files in the selected folder.</p>
                <div class="state-actions">
                    <button class="btn btn-primary" onclick={changeFolder}><FolderOpen size={15} />Choose folder</button>
                </div>
            </div>
        {:else if visible.length === 0}
            <div class="state">
                <div class="state-icon"><SearchX size={22} /></div>
                <h2>No matching flights</h2>
                <p>Try a different search or file type.</p>
                <div class="state-actions">
                    <button class="btn" onclick={() => { query = ""; typeFilter = "all"; }}>Clear filters</button>
                </div>
            </div>
        {:else}
            <ul class="rows">
                {#each visible as flight (flight.path)}
                    {@const isTelem = flight.extension === "telem"}
                    <li>
                        <button class="row" onclick={() => openFlight(flight)} title={flight.path}>
                            <div class="name-cell">
                                <span class="file-icon" class:telem={isTelem}>
                                    {#if isTelem}<FileText size={17} />{:else}<FileSpreadsheet size={17} />{/if}
                                </span>
                                <span class="name">{baseName(flight.name)}</span>
                            </div>
                            <span><span class="badge" class:badge-accent={isTelem}>{flight.extension}</span></span>
                            <span class="col-date muted tabular">
                                {#if flight.modifiedMs}
                                    {dateFormat.format(flight.modifiedMs)}
                                    <span class="time">{timeFormat.format(flight.modifiedMs)}</span>
                                {:else}-{/if}
                            </span>
                            <span class="col-size muted tabular">{formatBytes(flight.sizeBytes)}</span>
                            <span class="open"><span class="open-label">Open</span><ChevronRight size={16} /></span>
                        </button>
                    </li>
                {/each}
            </ul>
        {/if}
    </section>

    {#if !isLoading && !errorMessage && flights.length > 0}
        <p class="footnote">
            Showing {visible.length} of {flights.length}
        </p>
    {/if}
</div>

<style>
    #open-folder-btn {
        padding: 0.15rem;
        margin: 0;
        height: fit-content;
        width: fit-content;
    }

    .page {
        max-width: 1080px;
        margin: 0 auto;
        padding: 2rem 1.5rem 3rem;
    }

    /* Header */
    .page-header {
        display: flex;
        align-items: flex-end;
        justify-content: space-between;
        gap: 1rem;
        margin-bottom: 1.5rem;
    }

    h1 { font-size: 24px; }

    .subtitle {
        margin-top: 0.35rem;
        color: var(--text-muted);
        display: flex;
        align-items: center;
        flex-wrap: wrap;
        gap: 0.3rem;
    }

    .folder {
        display: inline-flex;
        align-items: center;
        gap: 0.3rem;
        max-width: 480px;
        padding: 0 0.45rem;
        height: 22px;
        border-radius: 6px;
        background: var(--surface-2);
        border: 1px solid var(--border);
        color: var(--text);
        font-size: 12px;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
    .folder :global(svg) { color: var(--accent); flex-shrink: 0; }

    .header-actions { display: flex; gap: 0.5rem; flex-shrink: 0; }

    .spinning { display: inline-flex; animation: spin 0.9s linear infinite; }

    /* Toolbar */
    .toolbar {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        margin-bottom: 0.9rem;
        flex-wrap: wrap;
    }

    .search, .sort {
        position: relative;
        display: flex;
        align-items: center;
    }
    .search { flex: 1; min-width: 200px; }
    .search :global(svg), .sort :global(svg) {
        position: absolute;
        left: 0.65rem;
        color: var(--text-subtle);
        pointer-events: none;
    }
    .search .input { width: 100%; padding-left: 2.1rem; }
    .sort .select { padding-left: 2.1rem; }

    .count {
        font-size: 11px;
        color: var(--text-subtle);
        font-variant-numeric: tabular-nums;
    }
    .segmented button.active .count { color: var(--accent-text); }

    /* List */
    .list-card { overflow: hidden; }

    .list-head, .row {
        display: grid;
        grid-template-columns: minmax(0, 1fr) 90px 170px 80px 72px;
        align-items: center;
        gap: 1rem;
        padding: 0 1rem;
    }

    .list-head {
        height: 38px;
        background: var(--surface-2);
        border-bottom: 1px solid var(--border);
        font-size: 11px;
        font-weight: 650;
        letter-spacing: 0.06em;
        text-transform: uppercase;
        color: var(--text-subtle);
    }

    .rows { list-style: none; margin: 0; padding: 0; }
    .rows li + li { border-top: 1px solid var(--border); }

    .row {
        width: 100%;
        min-height: 58px;
        border: none;
        background: transparent;
        text-align: left;
        color: var(--text);
        position: relative;
        transition: background-color 0.12s;
    }
    .row::before {
        content: "";
        position: absolute;
        left: 0;
        top: 10px;
        bottom: 10px;
        width: 3px;
        border-radius: 0 3px 3px 0;
        background: var(--accent);
        opacity: 0;
        transition: opacity 0.12s;
    }
    .row:hover { background: var(--surface-2); }
    .row:hover::before, .row:focus-visible::before { opacity: 1; }
    .row:focus-visible { outline: none; background: var(--accent-soft); }

    .name-cell {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        min-width: 0;
    }

    .file-icon {
        display: grid;
        place-items: center;
        width: 34px;
        height: 34px;
        flex-shrink: 0;
        border-radius: 8px;
        background: var(--surface-3);
        color: var(--text-muted);
    }
    .file-icon.telem { background: var(--accent-soft); color: var(--accent-text); }

    .name {
        font-weight: 600;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .muted { color: var(--text-muted); font-size: 13px; }
    .time { color: var(--text-subtle); margin-left: 0.35rem; }

    .open {
        display: inline-flex;
        align-items: center;
        justify-content: flex-end;
        gap: 0.15rem;
        color: var(--text-subtle);
        font-size: 12.5px;
        font-weight: 600;
    }
    .open-label { opacity: 0; transform: translateX(4px); transition: opacity 0.12s, transform 0.12s; }
    .row:hover .open, .row:focus-visible .open { color: var(--accent-text); }
    .row:hover .open-label, .row:focus-visible .open-label { opacity: 1; transform: none; }

    /* Skeleton */
    .skeleton-row { min-height: 58px; }
    .skeleton-row + .skeleton-row { border-top: 1px solid var(--border); }
    .sk {
        display: block;
        border-radius: 6px;
        background: linear-gradient(90deg, var(--surface-2) 0%, var(--surface-3) 50%, var(--surface-2) 100%);
        background-size: 800px 100%;
        animation: shimmer 1.4s linear infinite;
    }
    .sk-icon { width: 34px; height: 34px; border-radius: 8px; }
    .sk-text { width: 45%; height: 12px; }
    .sk-badge { width: 46px; height: 18px; border-radius: 999px; }
    .sk-short { width: 110px; height: 12px; }
    .sk-tiny { width: 50px; height: 12px; }

    /* Empty / error states */
    .state {
        display: flex;
        flex-direction: column;
        align-items: center;
        text-align: center;
        padding: 3.5rem 1.5rem;
        gap: 0.4rem;
    }
    .state h2 { font-size: 16px; margin-top: 0.5rem; }
    .state p { color: var(--text-muted); max-width: 420px; }
    .state-icon {
        display: grid;
        place-items: center;
        width: 48px;
        height: 48px;
        border-radius: 12px;
        background: var(--accent-soft);
        color: var(--accent-text);
    }
    .state-icon.danger { background: var(--danger-soft); color: var(--danger); }
    .state-actions { display: flex; gap: 0.5rem; margin-top: 1rem; }

    .footnote {
        margin-top: 0.75rem;
        font-size: 12px;
        color: var(--text-subtle);
        text-align: right;
    }

    @media (max-width: 760px) {
        .list-head, .row { grid-template-columns: minmax(0, 1fr) 70px 40px; }
        .col-date, .col-size { display: none; }
        .open-label { display: none; }
        .page-header { flex-direction: column; align-items: flex-start; }
    }
</style>
