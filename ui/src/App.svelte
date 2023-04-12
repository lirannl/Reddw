<script lang="ts">
  import type { AppConfig } from "$rs/AppConfig";
  import { invoke } from "@tauri-apps/api";
  import Config from "./Config.svelte";
  import type { Wallpaper } from "$rs/Wallpaper";
  const initConfigPromise = invoke<AppConfig>("get_config");
  const queuePromise = invoke<Wallpaper[]>("get_queue");
</script>

<main class="w-screen h-screen">
  {#await initConfigPromise}
    <div>Loading...</div>
  {:then initConfig}
    <Config config={initConfig} />
  {/await}
  <button-group class="btn-group">
    <button class="btn" on:click={() => invoke("cache_queue")}>Cache upcoming</button>
    <button class="btn" on:click={() => invoke("update_wallpaper")}>Update wallpaper</button
    >
    <button class="btn btn-warning" on:click={() => invoke("exit")}>Quit</button>
  </button-group>
</main>
