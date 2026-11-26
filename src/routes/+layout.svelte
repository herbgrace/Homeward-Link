<script lang="ts">
    import "@fontsource-variable/inter";
    import NavigationBar from "$lib/components/NavigationBar.svelte";

    let { children } = $props();
</script>

<div class="app-shell">
    <NavigationBar />
    <div class="app-content">
        {@render children()}
    </div>
</div>

<style>
    .app-shell {
        display: flex;
        flex-direction: column;
        height: 100vh;
        overflow: hidden;
    }

    .app-content {
        flex: 1;
        min-height: 0;
        overflow: auto;
        position: relative;
    }
</style>
