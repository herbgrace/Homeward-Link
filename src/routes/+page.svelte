<script lang="ts">
  import '../defaultPallette.css'
  import NavigationBar from '../lib/NavigationBar.svelte';

  let fileInput: HTMLInputElement | null = null;
  let selectedFiles: FileList | null = $state(null);

  function triggerFileInput(event: Event) {
    fileInput?.click();
  }

  function handleFileChange(event: Event) {
    const target = event.target as HTMLInputElement;
    selectedFiles = target.files;
  }
</script>

<main class="container">
<NavigationBar />
  <h1>Welcome to Homeward Link!</h1>

  <input 
    type="file"
    accept=".csv, .json"
    bind:this={fileInput}
    onchange={handleFileChange}
    style="display: none"
    webkitdirectory
  /> 
  <button onclick={triggerFileInput}>
    Select a file
  </button>

  {#each selectedFiles as file}
    <p>File Name: {file.name}</p>
  {/each}
</main>