<script lang="ts">
  import VirtualList from "svelte-tiny-virtual-list";
  import type { Wallpaper } from "$rs/Wallpaper";
  import { bg_data, wp_list } from "./App.svelte";
  import { invoke } from "@tauri-apps/api";

  const itemCards: HTMLDivElement[] = [];
  let bg_opacity: number | undefined;
  bg_data.subscribe((d) => (bg_opacity = d?.bg_opacity));
  let queue: Wallpaper[];
  wp_list.subscribe((q) => {
    queue = q;
  });
  let data: string | undefined;
  let openIndex: number | undefined = undefined;
</script>

<carousel class="carousel carousel-center rounded-box">
  {#each queue as wallpaper, index}
    <div
      class="carousel-item card"
      style={`--tw-bg-opacity: ${bg_opacity};`}
      bind:this={itemCards[index]}
      on:mouseenter={() => {
        new Promise((resolve) => setTimeout(resolve, 500)).then(async () => {
          if (
            ![...document.querySelectorAll(":hover")].includes(itemCards[index])
          )
            return;
          const d = await invoke("get_wallpaper", { wallpaper });
          openIndex = index;
          data = `data:image;base64,${d}`;
        });
      }}
      on:mouseleave={() => {
        data = undefined;
        openIndex = undefined;
      }}
    >
      {#if data && openIndex === index}
        <div class="overflow-y-auto max-h-32">
          <img
            class="collapse-content object-cover"
            src={data}
            alt={wallpaper.name}
          />
        </div>
      {/if}
      {wallpaper.name}
      <button
        class="btn"
        on:click={() => {
          invoke("set_wallpaper", { wallpaper: queue[index] });
        }}>Set wallpaper</button
      >
    </div>
  {/each}
</carousel>
