<script lang="ts">
    interface Props {
        src: string;
        alt: string;
        ratio?: string;   // CSS aspect-ratio, e.g. "16 / 9"
        label?: string;   // text shown in the placeholder (defaults to alt)
    }

    let { src, alt, ratio = "16 / 9", label = alt }: Props = $props();

    let failed = $state(false);
    $effect(() => { src; failed = false; });   // retry if the src changes
</script>

{#if failed}
    <div class="placeholder" style:aspect-ratio={ratio}>
        <span>{label}</span>
    </div>
{:else}
    <img {src} {alt} style:aspect-ratio={ratio} onerror={() => (failed = true)} />
{/if}

<style>
    img, .placeholder {
        display: block;
        width: 100%;
        object-fit: cover;
        border-radius: 0.5rem;
        box-sizing: border-box;
    }
    .placeholder {
        display: grid;
        place-items: center;
        padding: 1rem;
        text-align: center;
        border: 2px dashed var(--accent-color, #888);
        color: var(--accent-color, #888);
        background: rgba(128, 128, 128, 0.08);
        font-size: 0.9rem;
    }
</style>