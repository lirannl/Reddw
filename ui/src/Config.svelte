<script lang="ts">
    import type { AppConfig } from "$rs/AppConfig";
    import type { Source } from "$rs/Source";
    import { invoke } from "@tauri-apps/api";
    import { sourceTypes } from "./types/source";
    import { bg_data } from "./App.svelte";

    export let config: AppConfig;
    let lightness: number;
    bg_data.subscribe((d) => {
        if (d?.lightness) lightness = d.lightness;
    });

    const getSrcTypes = (cfg: AppConfig) =>
        cfg.sources.map((src) => Object.keys(src)[0] as keyof Source);

    let srcTypes: (keyof Source)[] = getSrcTypes(config);

    const onFormChange = () => {
        invoke("set_config", { appConfig: config });
    };

    const removeSource = (i: number) => {
        const sourcesCopy = config.sources;
        sourcesCopy.splice(i, 1);
        config.sources = sourcesCopy;
        const srcTypesCopy = srcTypes;
        srcTypesCopy.splice(i, 1);
        srcTypes = srcTypesCopy;
        onFormChange();
    };
</script>

<form
    on:change={onFormChange}
    class={`card card-compact m-auto bg-base-100 bg-opacity-${
        lightness > 100 ? 50 : 25
    } backdrop-blur-lg shadow-xl`}
>
    <card-body class="card-body">
        <h2 class="card-title">Configuration</h2>
        <collapse class="collapse collapse-arrow">
            <input type="checkbox" />
            <div class="collapse-title text-xl font-medium">Sources</div>
            <div class="collapse-content">
                {#each config.sources as source, i}
                    <div class="input-group max-w-xs">
                        <select
                            class="select select-primary"
                            bind:value={srcTypes[i]}
                        >
                            {#each sourceTypes as sourceType}
                                <option value={sourceType}>{sourceType}</option>
                            {/each}
                        </select>
                        <input
                            type="text"
                            bind:value={source[srcTypes[i]]}
                            class="input input-bordered"
                        />
                        <button
                            class="btn btn-secondary"
                            on:click={() => removeSource(i)}>-</button
                        >
                    </div>
                {/each}
                <button
                    class="btn"
                    on:click={(e) => {
                        e.preventDefault();
                        config.sources = [
                            ...config.sources,
                            { [sourceTypes[0]]: "" },
                        ];
                        srcTypes = [...srcTypes, sourceTypes[0]];
                    }}>Add source</button
                >
            </div>
        </collapse>
        <div class="grid grid-cols-3 gap-4">
            <form-control class="form-control">
                <label for="allow_nsfw" class="label cursor-pointer">
                    <span class="label-text">Allow NSFW</span>
                </label>
                <input
                    id="allow_nsfw"
                    type="checkbox"
                    bind:checked={config.allow_nsfw}
                    class="checkbox border-base-content border-opacity-75"
                />
            </form-control>
            <form-control class="form-control">
                <label for="display_background" class="label cursor-pointer">
                    <span class="label-text">Display background in app</span>
                </label>
                <input
                    id="display_background"
                    type="checkbox"
                    bind:checked={config.display_background}
                    class="checkbox border-base-content border-opacity-75"
                />
            </form-control>
            <form-control class="form-control">
                <label for="theme" class="label">
                    <span class="label-text">Theme</span>
                </label>
                <input
                    id="theme"
                    type="text"
                    bind:value={config.theme}
                    class="input input-bordered w-full max-w-xs"
                />
            </form-control>
            <form-control class="form-control">
                <label for="updateInterval" class="label">
                    <span class="label-text">Interval</span>
                </label>
                <div class="input-group">
                    <input
                        id="updateInterval"
                        type="text"
                        value={config.interval.secs +
                            config.interval.nanos / 1000000000}
                        on:change={(e) => {
                            const val = parseFloat(e.currentTarget.value);
                            config.interval.secs = Math.floor(val);
                            config.interval.nanos = Math.floor(
                                (val - config.interval.secs) * 1000000000
                            );
                        }}
                        class="input input-bordered w-full max-w-xs"
                    />
                    <span>Seconds</span>
                </div>
            </form-control>
            <form-control class="form-control">
                <label for="folder" class="label">
                    <span class="label-text">Cache directory</span>
                </label>
                <div class="input-group">
                    <input
                        id="folder"
                        type="text"
                        bind:value={config.cache_dir}
                        class="input input-bordered w-full max-w-xs"
                    />
                    <button
                        class="btn"
                        on:click={async (e) => {
                            e.preventDefault();
                            config.cache_dir = await invoke("select_folder");
                            // Manually trigger form change (since an update is done synthetically)
                            onFormChange();
                        }}
                        ><svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="24"
                            height="24"
                            fill={getComputedStyle(document.body).color}
                            viewBox="0 0 24 24"
                            ><path
                                d="M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"
                            /></svg
                        ></button
                    >
                </div>
            </form-control>
            <form-control class="form-control">
                <label for="history_amount" class="label">
                    <span class="label-text">Max items in history</span>
                </label>
                <input
                    id="history_amount"
                    type="text"
                    bind:value={config.history_amount}
                    class="input input-bordered w-full max-w-xs"
                />
            </form-control>
            <form-control class="form-control">
                <label for="cache_size" class="label">
                    <span class="label-text">Max cache size</span>
                </label>
                <div class="input-group">
                    <input
                        id="cache_size"
                        type="text"
                        bind:value={config.cache_size}
                        class="input input-bordered w-full max-w-xs"
                    />
                    <span>MB</span>
                </div>
            </form-control>
        </div>
    </card-body>
</form>
