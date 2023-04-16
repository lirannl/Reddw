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

  import { onDestroy, onMount } from "svelte";
  import { reactToAppConfig, updateAppWallpaper } from "./misc";
  import { listen } from "@tauri-apps/api/event";

  export let initConfig: AppConfig;
  let main: HTMLElement;
  configuration.set(initConfig);
  let config: AppConfig | undefined;
  let queue: Wallpaper[] | undefined;

  const unlistens: (() => unknown)[] = [];

  onMount(() => {
    if (queue && config?.display_background) updateAppWallpaper(queue, main);
  });

  listen<[AppConfig, AppConfig]>(
    "config_changed",
    ({ payload: [oldConfig, newConfig] }) => {
      if (config && queue) reactToAppConfig(oldConfig, newConfig, queue, main);
      configuration.set(newConfig);
      config = newConfig;
    }
  ).then((unlisten) => {
    unlistens.push(unlisten);
  });
  configuration.subscribe((c) => {
    // if (config && queue) reactToAppConfig(config, c, queue, main);
    config = c;
  });

  listen<Wallpaper>("wallpaper_updated", async ({ payload }) => {
    if (!config?.display_background) return;
    const data = await invoke<string>("get_wallpaper", {
      wallpaper: payload,
    });
    main.style.backgroundImage = `url(data:image;base64,${data})`;
  }).then((unlisten) => {
    unlistens.push(unlisten);
  });

  wp_list.subscribe(async (l) => {
    queue = l;
  });

  export let initQueue: Wallpaper[];
  wp_list.set(initQueue);

  onDestroy(() => {
    unlistens.forEach((unlisten) => unlisten());
  });
</script>

<main class="w-screen h-screen p-2 space-y-2" bind:this={main}>
  {#if config}
    <Config {config} />
  {/if}
  <div class="backdrop-blur w-fit">
    <button-group class="btn-group">
      <button class="btn bg-opacity-60" on:click={() => invoke("cache_queue")}
        >Cache upcoming</button
      >
      <button
        class="btn bg-opacity-60"
        on:click={() => invoke("update_wallpaper")}>Update wallpaper</button
      >
      <button
        class="btn btn-warning bg-opacity-60"
        on:click={() => invoke("exit")}>Quit</button
      >
    </button-group>
  </div>
</main>
