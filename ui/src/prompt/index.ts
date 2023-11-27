import { dialog_data_model, dialog_data_output, dialog_data_show } from "../App.svelte";

export const promptUser = <V, Model extends Record<string, DialogFieldType<V>>>(model: Model) => new Promise<{
    [K in keyof Model]:
    (DialogFieldTypeMap<V>)[Model[K][0]]
}>(async (resolve, _) => {
    dialog_data_model.set(model);
    dialog_data_show.set(true);
    let out: any;
    const subCancel = dialog_data_output.subscribe(o => out = o);
    dialog_data_show.subscribe(s => {
        if (!s) {
            subCancel();
            resolve(out);
        }
    });
});

export type DialogFieldType<V = unknown> = ["string"] | ["number"] |
["multiSelect", V extends Record<string, unknown> ? V : Record<string, unknown>] |
["singleSelect", V extends Record<string, unknown> ? V : Record<string, unknown>];

export type DialogFieldTypeMap<V> = {
    string: string,
    number: number,
    multiSelect: V extends Record<string, unknown> ? (V[keyof V])[] : never,
    singleSelect: V extends Record<string, unknown> ? V[keyof V] : never,
};