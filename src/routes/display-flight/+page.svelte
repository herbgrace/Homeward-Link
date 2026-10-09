<script lang="ts">
import { page } from "$app/state";
import { goto } from "$app/navigation";

let filePath = $derived(page.url.searchParams.get('filePath'));

</script>

<main class="container">
<div class="header-bar">
    <button 
    class="return-button"
    onclick={() => goto("/previous-flights")}
    >
        Return
    </button>

    <h3>{filePath}</h3>
</div>
<p>You are no longer trapped here...</p>
<p>But I display the data here</p>
</main>

<style>
.header-bar {
    display: flex;
    align-items: center;
    margin-top: auto;
    /* justify-items: flex-start; */
}

button {
    display: flex;
    width: fit-content;
    padding-left: auto;
}

h3 {
    align-self: center;
    display: flex;
    margin-left: 1rem;
}
</style>