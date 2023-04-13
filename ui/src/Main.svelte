<script lang="ts">
    import type { AppConfig } from "$rs/AppConfig";
    import type { Wallpaper } from "$rs/Wallpaper";
    import { invoke } from "@tauri-apps/api";
    import App from "./App.svelte";

    const initAppDataPromise = Promise.all([
        invoke<AppConfig>("get_config"),
        invoke<Wallpaper[]>("get_queue"),
    ]);
</script>

{#await initAppDataPromise}
    <div class="radial-progress" />
{:then [initConfig, initQueue]}
    <App {...{ initConfig, initQueue }} />
{/await}
