<script lang="ts">
    import type { AppConfig } from "$rs/AppConfig";
    import type { Wallpaper } from "$rs/Wallpaper";
    import { invoke } from "@tauri-apps/api";
    import App from "./App.svelte";
    import type { AppSourcePlugins } from "./misc";

    const initAppDataPromise = Promise.all([
        invoke<AppConfig>("get_config"),
        invoke<Wallpaper[]>("get_queue"),
        invoke<AppSourcePlugins>("query_available_sources"),
    ]);
</script>

{#await initAppDataPromise}
    <div class="radial-progress" />
{:then [initConfig, initQueue, initSourcePlugins]}
    <App {...{ initConfig, initQueue, initSourcePlugins }} />
{/await}
