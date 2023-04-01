<script lang="ts">
    import type { AppConfig } from "$rs/AppConfig";
    import type { Source } from "$rs/Source";
    import { invoke } from "@tauri-apps/api";
    import { listen } from "@tauri-apps/api/event";
    import { sourceTypes } from "./types/source";

    export let config: AppConfig;
    const getSrcTypes = (cfg: AppConfig) =>
        cfg.sources.map((src) => Object.keys(src)[0] as keyof Source);

    let srcTypes: (keyof Source)[] = getSrcTypes(config);
    listen<AppConfig>("config_changed", ({ payload }) => {
        srcTypes = getSrcTypes(payload);
        config = payload;
    });

    const onFormChange = (
        e: Event & { currentTarget: EventTarget & HTMLFormElement }
    ) => {
        invoke("set_config", { appConfig: config });
    };

    const removeSource = (i: number) => {
        config.sources.splice(i, 1);
        srcTypes.splice(i, 1);
        onFormChange({ currentTarget: null } as any);
    };
</script>

<form on:change={onFormChange}>
    <label>
        Allow NSFW
        <input type="checkbox" bind:checked={config.allow_nsfw} />
    </label>
    <div>
        Sources:
        {#each config.sources as source, i}
            <div>
                <select bind:value={srcTypes[i]}>
                    {#each sourceTypes as sourceType}
                        <option value={sourceType}>{sourceType}</option>
                    {/each}
                </select>
                <input type="text" bind:value={source[srcTypes[i]]} />
                <button on:click={() => removeSource(i)}>-</button>
            </div>
        {/each}
        <button
            on:click={(e) => {
                e.preventDefault();
                config.sources = [...config.sources, { [sourceTypes[0]]: "" }];
                srcTypes = [...srcTypes, sourceTypes[0]];
            }}>Add source</button
        >
    </div>
    <label>
        Update interval:
        <input
            value={config.interval.secs + config.interval.nanos / 1000000000}
            on:change={(e) => {
                const val = parseFloat(e.currentTarget.value);
                config.interval.secs = Math.floor(val);
                config.interval.nanos = Math.floor(
                    (val - config.interval.secs) * 1000000000
                );
            }}
        />
    </label><br />
    <label>
        Cache directory: <input bind:value={config.cache_dir} />
        <button
            on:click={async (e) => {
                e.preventDefault();
                config.cache_dir = await invoke("select_folder");
            }}>Select folder</button
        >
    </label><br />
    <label>
        Max items in history:
        <input bind:value={config.history_amount} />
    </label><br />
    <label>
        Max cache size:
        <input type="number" bind:value={config.cache_size} />
    </label>
</form>
