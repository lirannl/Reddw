<script lang="ts" context="module">
  import { writable } from "svelte/store";
  export const configuration = writable({} as AppConfig);
  export const wp_list = writable([] as Wallpaper[]);
</script>

<script lang="ts">
  import { invoke } from "@tauri-apps/api";
  import Config from "./Config.svelte";
  import type { AppConfig } from "$rs/AppConfig";
  import type { Wallpaper } from "$rs/Wallpaper";

  let config: AppConfig;
  export let initConfig: AppConfig;
  configuration.set(initConfig);
  configuration.subscribe((c) => (config = c));
  export let initQueue: Wallpaper[];
  wp_list.subscribe(async (l) => {
    const current = l.find((w) => w.was_set);
    if (current)
      console.log(await invoke("get_wallpaper_path", { wallpaper: current }));
  });
  wp_list.set(initQueue);
</script>

<main class="w-screen h-screen">
  <Config />
  <button-group class="btn-group">
    <button class="btn" on:click={() => invoke("cache_queue")}
      >Cache upcoming</button
    >
    <button class="btn" on:click={() => invoke("update_wallpaper")}
      >Update wallpaper</button
    >
    <button class="btn btn-warning" on:click={() => invoke("exit")}>Quit</button
    >
  </button-group>
</main>
