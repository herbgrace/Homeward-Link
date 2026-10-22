<script lang="ts">
import { getLocalTimeZone, today, type CalendarDate } from "@internationalized/date";

import NavigationBar from '$lib/components/NavigationBar.svelte';
import { Calendar } from "$lib/components/ui/calendar/index.js";

let value = $state<CalendarDate | undefined>(today(getLocalTimeZone()));
</script>

<main class="container">
<title>Calendar</title>
<NavigationBar />
<div class="calendar-wrapper">
    <Calendar 
        type="single" 
        bind:value 
        class="rounded-lg border" 
        captionLayout="dropdown" 
    />
</div>
</main>

<style>
    /* target and neutralize generic button styles inside the calendar */
    :global(.calendar-wrapper button) {
        padding: 0 !important;
        font-size: unset !important;
        font-weight: unset !important;
        box-shadow: none !important;
        border: none !important;
        background-color: transparent !important;
        /* width: 100%; */
        height: 100%;
    }

    .calendar-wrapper {
        display: flex;
        justify-content: center;
        flex-direction: column;
        align-content: center;
        align-items: center;
        background-color: var(--secondary-color);
        padding: 0.75rem;
        width:fit-content;
        align-self: center;
        border-radius: 1rem;
    }
</style>