import { For, createEffect, createResource, createSignal, on } from "solid-js";
import { AiOutlineMinus, AiOutlinePlus } from "solid-icons/ai"
import { appConfig, updateAppConfig } from "./context/config";
import { invoke } from "./overrides";

export default () => {
    let sourceConfig: HTMLElement = undefined as any;
    let propagate_update = false;
    createEffect(on(appConfig, (appConfig) => { if (propagate_update) { propagate_update = false; console.log(appConfig); } }));
    const [source_plugins, /*{ mutate, refetch }*/] = createResource(() => invoke<string[]>("query_available_source_plugins"));
    const [newSource, setNewSource] = createSignal<string | undefined>();
    const [newInstance, setNewInstance] = createSignal("");
    const [selectedSource, selectSource] = createSignal<[string, string] | undefined>();
    // const [config, updateConfig] = useConfig();
    const loadPlugin = async (name: string) => {
        const configElement = document.createElement(`${name.toLowerCase()}-config`);
        const files = await invoke<Record<string, number[]>>("load_plugin_ui", { plugin: name });
        if (!files) return;
        const bundledData = await Promise.all(Object.entries(files).map(async ([name, data]) => {
            if (name.endsWith(".js")) {
                const asset = `data:text/javascript;charset=utf-8,${encodeURIComponent(String.fromCharCode(...data))}`;
                await import(/* @vite-ignore */ asset);
            }
            else if (name.endsWith(".css")) {
                return await new CSSStyleSheet().replace(String.fromCharCode(...data));
            }
        }));
        sourceConfig.replaceWith(configElement);
        sourceConfig = configElement;
        bundledData.forEach(item => {
            if (item instanceof CSSStyleSheet)
                sourceConfig.shadowRoot?.adoptedStyleSheets.push(item);
        });
        // sourceConfig.addEventListener("change", console.log);
        sourceConfig.addEventListener("input", e => {
            if (!(e instanceof CustomEvent)) return;
            console.log(selectedSource(), e.detail)
        });
    };
    const removeSource = (source_instance: string) => () => {
        propagate_update = true;
        updateAppConfig(prev => ({
            ...prev, sources: Object.fromEntries(
                Object.entries(prev.sources).filter(([k, _]) => k != source_instance)
            )
        }));
    }
    const addSource = () => {
        const newSourceValue = newSource();
        updateAppConfig(prev => ({ ...prev, sources: Object.fromEntries([...Object.entries(prev.sources), [`${newSourceValue}_${newInstance()}`, {}]]) }));
        setNewSource(); setNewInstance("");
        if (newSourceValue) loadPlugin(newSourceValue);
    }
    return <div class="card">
        <form class="card-body" onSubmit={e => { e.preventDefault() }}>
            <div class="collapse bg-base-200">
                <input type="checkbox" checked={true} />
                <div class="collapse-title text-xl font-medium">
                    Sources
                </div>
                <div class="collapse-content">
                    <For each={Object.keys(appConfig().sources)}>{source_instance => {
                        const [source, instance] = source_instance.split("_");
                        return <div classList={{
                            "flex": true, "gap-1": true,
                            "items-center": true, "justify-between": true,
                            "bg-secondary": [source, instance].equals(selectedSource() ?? [])
                        }}>
                            <div class="btn" onClick={() => { selectSource([source, instance]); loadPlugin(source) }}>
                                <div class="bg-neutral rounded-md px-1 text-xl">{source}</div>
                                <div class="bg-neutral rounded-md px-1 text-xl">{instance}</div>
                            </div>
                            <div class="btn bg-neutral" onClick={removeSource(source_instance)}>
                                Remove<AiOutlineMinus />
                            </div>
                        </div>
                    }}</For>
                    <div class="input overflow-y-hidden flex items-center">
                        <select class="select outline-none" onInput={(e) => setNewSource(e.target.value)} value={newSource()}>
                            <option value={undefined}>Select source</option>
                            <For each={source_plugins()}>{source_plugin =>
                                <option>{source_plugin}</option>
                            }</For>
                        </select>
                        <input class="b-0 outline-none bg-transparent w-full" onInput={e => setNewInstance(e.target.value)} value={newInstance() ?? ""} />
                        <div onClick={() => { const s = newSource(); if (s) loadPlugin(s) }} classList={{
                            btn: true, "btn-disabled": !(newSource() && newInstance())
                        }}><AiOutlinePlus title="Add" onClick={addSource} /></div>
                    </div>
                    {//@ts-ignore
                        <div ref={sourceConfig} />}
                </div>
            </div>
        </form>
    </div>
}