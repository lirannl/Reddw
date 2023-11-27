<script lang="ts">
    import type { DialogFieldType } from ".";
    import {
        dialog_data_model,
        dialog_data_output,
        dialog_data_show,
    } from "../App.svelte";

    let dataModel: Record<string, DialogFieldType> = {};
    dialog_data_model.subscribe((m) => (dataModel = m));
    let output: Record<string, unknown> = {};
    dialog_data_output.subscribe((o) => (output = o));
    let showModal = false;
    dialog_data_show.subscribe((s) => (showModal = s));
    let dialog: HTMLDialogElement;
    $: if (dialog && showModal) dialog.showModal();
</script>

<dialog
    class="dialog"
    bind:this={dialog}
    on:close={() => {
        dialog_data_show.set(false);
        dialog.close();
    }}
    on:keypress={({ key }) => {
        if (key === "Escape") showModal = false;
    }}
>
    {#each Object.entries(dataModel) as [key, [type, extraData]]}
        <label
            >{key}
            {#if type === "string"}
                <input type="text" bind:value={output[key]} />
            {/if}
            {#if type === "number"}
                <input type="number" bind:value={output[key]} />
            {/if}
            {#if type === "singleSelect" && extraData}
                <select bind:value={output[key]}>
                    {#each Object.keys(extraData) as k}
                        <option value={extraData[k]}>{k}</option>
                    {/each}
                </select>
            {/if}
        </label>
    {/each}
    <button
        class="btn"
        on:click={() => {
            dialog_data_show.set(false);
            dialog.close();
        }}>Ok</button
    >
</dialog>
