<script lang="ts">
    import type { Wallpaper } from "$rs/Wallpaper";
    import { wp_list } from "./App.svelte";
    let queue: Wallpaper[];
    wp_list.subscribe((q) => {
        queue = q;
    });
    $: wp_list.set(queue);
</script>
