import { For, createEffect, createResource, createSignal, on } from "solid-js";
import { AiOutlineMinus, AiOutlinePlus } from "solid-icons/ai"
import { appConfig, updateAppConfig } from "./context/config";
import { invoke } from "./overrides";

export default () => {
    let sourceConfig: HTMLElement = undefined as any;
    let propagate_update = false;
    const [selectedSource, selectSource] = createSignal<[string, string] | undefined>();
    createEffect(on(appConfig, (appConfig) => {
        if (propagate_update) {
            propagate_update = false;
        }
        const plugin_instance = selectedSource()?.join("_");
        if (plugin_instance) sourceConfig.setAttribute("value", JSON.stringify(appConfig.sources[plugin_instance]));
    }));
    const [source_plugins] = createResource(() => invoke<string[]>("query_available_source_plugins"));
    const [newSource, setNewSource] = createSignal<string | undefined>();
    const [newInstance, setNewInstance] = createSignal("");
    const loadPlugin = async (name: string) => {
        const configElement = document.createElement(`${name.toLowerCase()}-config`);
        const files = await invoke<Record<string, number[]>>("load_plugin_ui", { plugin: name });
        if (!files) return;
        const plugin_instance = selectedSource()?.join("_");
        if (plugin_instance && appConfig().sources[plugin_instance]) configElement.setAttribute("value", JSON.stringify(appConfig().sources[plugin_instance]));
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
        sourceConfig.addEventListener("input", async e => {
            if (!(e instanceof CustomEvent) || !plugin_instance) return;
            const currentConfig = appConfig();
            await invoke("set_config", { appConfig: { ...currentConfig, sources: { ...currentConfig.sources, [plugin_instance]: e.detail } } })
        });
    };

    const removeSource = (source_instance: string) => () => {
        propagate_update = true;
        updateAppConfig(prev => ({
            ...prev, sources: Object.fromEntries(
                Object.entries(prev.sources).filter(([k, _]) => k != source_instance)
            )
        }));
    };

    const addSource = () => {
        const newSourceValue = newSource();
        updateAppConfig(prev => ({ ...prev, sources: Object.fromEntries([...Object.entries(prev.sources), [`${newSourceValue}_${newInstance()}`, {}]]) }));
        setNewSource(); setNewInstance("");
        if (newSourceValue) loadPlugin(newSourceValue);
    };

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
                            "join": true,
                            "bg-neutral": [source, instance].equals(selectedSource() ?? [])
                        }}>
                            <div class="btn join-item w-full" onClick={() => { selectSource([source, instance]); loadPlugin(source) }}>
                                <div class="bg-neutral rounded-md px-1 text-xl">{source}</div>
                                <div class="bg-neutral rounded-md px-1 text-xl">{instance}</div>
                            </div>
                            <div class="btn bg-secondary bg-opacity-50 join-item" onClick={removeSource(source_instance)}>
                                Remove<AiOutlineMinus />
                            </div>
                        </div>
                    }}</For>
                    <div class="join w-full" onKeyPress={({ code }) => { if (code === "Enter" && newSource() && newInstance()) addSource() }}>
                        <select class="select join-item" onInput={(e) => setNewSource(e.target.value)} value={newSource()}>
                            <option value={undefined}>Select source</option>
                            <For each={source_plugins()}>{source_plugin =>
                                <option>{source_plugin}</option>
                            }</For>
                        </select>
                        <input class="input bg-neutral join-item" onInput={e => setNewInstance(e.target.value)} value={newInstance() ?? ""} />
                        <div onClick={() => { const s = newSource(); if (s) loadPlugin(s) }} classList={{
                            "join-item": true, btn: true, "btn-disabled": !(newSource() && newInstance())
                        }}><AiOutlinePlus title="Add" onClick={addSource} /></div>
                    </div>
                    {//@ts-ignore
                        <div ref={sourceConfig} />}
                </div>
            </div>
        </form>
    </div>;
};