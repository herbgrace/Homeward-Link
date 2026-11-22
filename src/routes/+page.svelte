<script lang="ts">
    import { FolderClock, Radio, ArrowRight } from "@lucide/svelte";
    import ImageSlot from "$lib/components/ImageSlot.svelte";

    const features = [
        {
            title: "Replay past flights",
            text: "Open a .telem or .csv log and watch the flight play back with all of its included telemetry.",
            img: "/img/home/replay.png",
        },
        {
            title: "Track live",
            text: "Plug in a wireless receiver and see telemetry the moment it arrives.",
            img: "/img/home/live.png",
        },
        {
            title: "See the rocket move",
            text: "A 3D model and flight instruments show orientation, altitude and vertical speed as the data plays.",
            img: "/img/home/instruments.png",
        },
    ];

    const hardware = [
        { name: "TeleDongle", detail: "Ground station", href: "https://altusmetrum.org/TeleDongle/", img: "/img/home/teledongle.png" },
        { name: "TeleMega", detail: "Flight computer", href: "https://altusmetrum.org/TeleMega/", img: "/img/home/telemega.png" },
        { name: "ESP32 + MPU6050", detail: "Test board", href: "https://www.espressif.com/en/products/socs/esp32", img: "/img/home/esp32.png" },
    ];

    const steps = [
        { title: "Connect", text: "Connect your telemetry receiver, or pick a saved flight log." },
        { title: "Choose", text: "Choose Live Flight or open a flight from Previous Flights." },
        { title: "Follow", text: "Press Play and follow the data." },
    ];
</script>

<svelte:head>
    <title>Homeward Link</title>
</svelte:head>

<div class="page">
    <section class="hero">
        <div class="hero-text">
            <span class="eyebrow">Telemetry for Altus Metrum flight computers</span>
            <h1>Follow your rocket from the <span class="accent">pad</span> to the field.</h1>
            <p class="lead">
                Live telemetry and flight replay in one place — with a 3D view of the rocket and
                cockpit-style instruments.
            </p>
            <div class="cta">
                <a class="btn btn-primary btn-lg" href="/previous-flights"><FolderClock size={16} />Open a previous flight</a>
                <a class="btn btn-lg" href="/live-flight"><Radio size={16} />Go live</a>
            </div>
        </div>
        <div class="hero-image">
            <ImageSlot src="/img/home/hero.png" alt="Homeward Link logo" ratio="4 / 5" label="Hero image: Homeward Link Logo (4:5)" />
        </div>
    </section>

    <section>
        <h2 class="section-title">What it does</h2>
        <div class="feature-grid">
            {#each features as f}
                <article class="card feature">
                    <ImageSlot src={f.img} alt={f.title} ratio="16 / 11" label="{f.title} (16:11)" />
                    <div class="feature-body">
                        <h3>{f.title}</h3>
                        <p>{f.text}</p>
                    </div>
                </article>
            {/each}
        </div>
    </section>

    <div class="split">
        <section>
            <h2 class="section-title">Getting started</h2>
            <ol class="steps">
                {#each steps as s, i}
                    <li>
                        <span class="step-num">{i + 1}</span>
                        <div>
                            <strong>{s.title}</strong>
                            <p>{s.text}</p>
                        </div>
                    </li>
                {/each}
            </ol>
        </section>

        <section>
            <h2 class="section-title">Works with</h2>
            <div class="hardware">
                {#each hardware as h}
                    <a class="card hw" href={h.href} target="_blank" rel="noreferrer">
                        <div class="hw-img"><ImageSlot src={h.img} alt={h.name} ratio="1 / 1" label={h.name} /></div>
                        <div class="hw-text">
                            <strong>{h.name}</strong>
                            <span>{h.detail}</span>
                        </div>
                        <ArrowRight size={15} />
                    </a>
                {/each}
            </div>
            <p class="small">Saved flights: <code>.telem</code> and <code>.csv</code> files.</p>
        </section>
    </div>
</div>

<style>
    .page {
        max-width: 1120px;
        margin: 0 auto;
        padding: 3rem 1.5rem 4rem;
        display: flex;
        flex-direction: column;
        gap: 3.5rem;
    }

    /* Hero */
    .hero {
        display: grid;
        grid-template-columns: minmax(0, 1fr) minmax(0, clamp(220px, 28vw, 340px));
        gap: 3rem;
        align-items: center;
    }

    .eyebrow {
        display: inline-block;
        padding: 0.25rem 0.65rem;
        border-radius: 999px;
        background: var(--accent-soft);
        color: var(--accent-text);
        font-size: 12px;
        font-weight: 600;
    }

    h1 {
        margin-top: 1rem;
        font-size: clamp(2rem, 4vw, 3rem);
        font-weight: 750;
        letter-spacing: -0.03em;
        line-height: 1.1;
    }

    .accent { color: var(--accent); }

    .lead {
        margin-top: 1rem;
        max-width: 540px;
        font-size: 16px;
        color: var(--text-muted);
        line-height: 1.6;
    }

    .cta { display: flex; gap: 0.6rem; margin-top: 1.75rem; flex-wrap: wrap; }
    .btn-lg { height: 40px; padding: 0 1.1rem; font-size: 14px; }
    .cta a:hover { text-decoration: none; }

    .hero-image :global(img), .hero-image :global(.placeholder) {
        border-radius: var(--radius-lg);
        box-shadow: var(--shadow-md);
    }

    /* Sections */
    .section-title {
        font-size: 13px;
        font-weight: 650;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        color: var(--text-subtle);
        margin-bottom: 1rem;
    }

    .feature-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
        gap: 1rem;
    }

    .feature { overflow: hidden; }
    .feature :global(img), .feature :global(.placeholder) { border-radius: 0; }
    .feature-body { padding: 1rem 1.1rem 1.2rem; }
    .feature h3 { font-size: 15px; margin-bottom: 0.35rem; }
    .feature p { color: var(--text-muted); line-height: 1.55; }

    .split {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
        gap: 3rem;
    }

    /* Steps */
    .steps { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 1rem; }
    .steps li { display: flex; gap: 0.9rem; align-items: flex-start; }
    .steps p { color: var(--text-muted); }
    .step-num {
        display: grid;
        place-items: center;
        flex-shrink: 0;
        width: 28px;
        height: 28px;
        border-radius: 50%;
        background: var(--accent-soft);
        color: var(--accent-text);
        font-weight: 700;
        font-size: 13px;
        border: 1px solid var(--accent-soft-strong);
    }

    /* Hardware */
    .hardware { display: flex; flex-direction: column; gap: 0.6rem; }
    .hw {
        display: flex;
        align-items: center;
        gap: 0.9rem;
        padding: 0.6rem 0.9rem 0.6rem 0.6rem;
        color: var(--text);
        transition: border-color 0.15s, background-color 0.15s;
    }
    .hw:hover { text-decoration: none; border-color: var(--accent); }
    .hw:hover > :global(svg) { color: var(--accent); transform: translateX(2px); }
    .hw > :global(svg) { color: var(--text-subtle); transition: transform 0.15s, color 0.15s; flex-shrink: 0; }
    .hw-img { width: 48px; flex-shrink: 0; }
    .hw-img :global(.placeholder) { font-size: 0; padding: 0; }
    .hw-text { flex: 1; display: flex; flex-direction: column; }
    .hw-text span { font-size: 12.5px; color: var(--text-muted); }

    .small { margin-top: 0.9rem; font-size: 12.5px; color: var(--text-subtle); }

    @media (max-width: 760px) {
        .hero { grid-template-columns: 1fr; }
        .hero-image { max-width: 280px; }
    }
</style>
