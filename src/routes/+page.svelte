<script lang="ts">
    import NavigationBar from "$lib/components/NavigationBar.svelte";
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

    const steps = [
        "Connect your telemetry receiver, or pick a saved flight log.",
        "Choose Live Flight or open a flight from Previous Flights.",
        "Press Play and follow the data.",
    ];
</script>

<svelte:head>
    <title>Homeward Link</title>
</svelte:head>



<main class="container">
  <div class="nav">
    <NavigationBar />
  </div>
  <!-- Hero -->
  <div class="main-content">
    <section class="hero">
        <div class="hero-text">
            <h1>Homeward Link</h1>
            <p class="tagline">Follow your rocket from the pad to the field.</p>
            <p>
                Live telemetry and flight replay in one place, built for Altus Metrum
                flight computers.
            </p>
        </div>
        <div class="hero-image">
            <ImageSlot src="/img/home/hero.png" alt="Homeward Link Logo" ratio="4 / 5" label="Hero image: Homeward Link Logo (4:5)" />
        </div>
    </section>

    <!-- Features -->
    <section>
        <h2>What it does</h2>
        <div class="cards">
            {#each features as f}
                <article class="card">
                    <ImageSlot src={f.img} alt={f.title} ratio="16 / 12" label="{f.title} (16:12)" />
                    <h3>{f.title}</h3>
                    <p>{f.text}</p>
                </article>
            {/each}
        </div>
    </section>

    <!-- Hardware & files -->
    <section class="works-with">
      <h2>Works with</h2>
      <div class="logos">
        <figure>
          <ImageSlot src="/img/home/teledongle.png" alt="TeleDongle" ratio="1 / 1" label="TeleDongle photo" />
          <figcaption><a href="https://altusmetrum.org/TeleDongle/" target="_blank">TeleDongle ground station</a></figcaption>
        </figure>
        <figure>
          <ImageSlot src="/img/home/telemega.png" alt="TeleMega" ratio="1 / 1" label="TeleMega photo" />
          <figcaption><a href="https://altusmetrum.org/TeleMega/" target="_blank">TeleMega flight computer</a></figcaption>
        </figure>
        <figure>
          <ImageSlot src="/img/home/esp32.png" alt="ESP32" ratio="1 / 1" label="ESP32 photo" />
          <figcaption><a href="https://www.espressif.com/en/products/socs/esp32" target="_blank">ESP32</a> 
            test board with an <a href="https://www.adafruit.com/product/3886" target="_blank">MPU6050 sensor</a>
          </figcaption>
        </figure>
      </div>
      <p class="small">Saved flights: <code>.telem</code> and <code>.csv</code> files.</p>
    </section>

    <!-- Getting started -->
    <section>
      <h2>Getting started</h2>
      <ol class="steps">
          {#each steps as s}
              <li>{s}</li>
          {/each}
      </ol>
    </section>
  </div>
</main>

<style>
  a {
    color: var(--text-color);
  }

  .main-content {
    padding-left: 3rem;
    padding-right: 3rem;
  }

  .container {
      max-width: 100%;
      margin: 0 auto;
      /* padding-left: 3rem;
      padding-right: 3rem; */
  }

  section { margin-top: 3rem; }

  h1 { 
    font-size: 2.6rem; 
    margin: 0 0 0.25rem; 
  }

  h2 { 
    margin: 0 0 1rem; 
  }

  h3 { 
    margin: 0.75rem 0 0.25rem; 
  }

  p  { 
    line-height: 1.5; 
  }

  /* Hero */
  .hero {
      display: grid;
      grid-template-columns: 1fr minmax(0, clamp(220px, 30vw, 360px));
      gap: 2rem;
      align-items: center;
  }

  .hero-image {
    width: 100%;
    max-width: clamp(220px, 30vw, 360px);
    justify-self: end;
  }

  .tagline {
    font-size: 1.2rem; 
    font-weight: 600; 
    margin: 0 0 1rem; 
  }

  /* Feature cards */
  .cards {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
      gap: 1.25rem;
  }
  .card {
      background: var(--secondary-color, #f4f4f4);
      border: 1px solid var(--accent-color, #ccc);
      border-radius: 0.6rem;
      padding: 1rem;
  }
  .card p { 
    margin: 0; 
  }

  /* Works with */
  .logos {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(140px, 200px));
      place-content: center;
      gap: 1.25rem;
  }

  figure { 
    margin: 0; 
    text-align: center; 
    width: fit-content; 
  }

  figcaption { 
    margin-top: 0.4rem; 
    font-size: 0.9rem; 
    overflow: none; 
  }

  .small { 
    font-size: 0.9rem; 
    opacity: 0.8; 
  }

  /* Steps */
  .steps { 
    padding-left: 1.25rem; 
    line-height: 1.8; 
    list-style: none;
  }

  @media (max-width: 720px) {
      .hero { grid-template-columns: 1fr; }
  }
</style>