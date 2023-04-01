<script lang="ts">
  import type { AppConfig } from "$rs/AppConfig";
  import { invoke } from "@tauri-apps/api";
  import Config from "./Config.svelte";
  const initConfigPromise = invoke<AppConfig>("get_config");
</script>

<main>
  {#await initConfigPromise}
    <div>Loading...</div>
  {:then initConfig}
    <Config config={initConfig} />
  {/await}
  <button on:click={() => invoke("cache_queue")}>Cache upcoming</button>
  <button on:click={() => invoke("update_wallpaper")}>Update wallpaper</button>
  <button on:click={() => invoke("quit")}>Quit</button>
  
</main>
