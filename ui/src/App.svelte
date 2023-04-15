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
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onDestroy } from "svelte";

  export let initConfig: AppConfig;
  let main: HTMLElement;
  configuration.set(initConfig);
  let config: AppConfig;
  let queue: Wallpaper[];

  const unlistens: (() => unknown)[] = [];

  listen<AppConfig>("config_changed", ({ payload }) => {
    configuration.set(payload);
  }).then(unlistens.push);

  listen<Wallpaper>("wallpaper_updated", async ({ payload }) => {
    if (!config.display_background) return;
    const data = await invoke<string>("get_wallpaper", {
      wallpaper: payload,
    });
    main.style["background-image"] = `url(data:image;base64,${data})`;
  }).then(unlistens.push);

  configuration.subscribe((c) => {
    config = c;
  });

  wp_list.subscribe(async (l) => {
    queue = l;
    const current = l.find((w) => w.was_set);
    if (current && config.display_background) {
      const data = await invoke<string>("get_wallpaper", {
        wallpaper: current,
      });
      main.style["background-image"] = `url(data:image;base64,${data})`;
    }
  });

  export let initQueue: Wallpaper[];
  wp_list.set(initQueue);

  onDestroy(() => {
    unlistens.forEach((unlisten) => unlisten());
  });
</script>

<main class="w-screen h-screen" bind:this={main}>
  <br />
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
