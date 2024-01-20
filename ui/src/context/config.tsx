import { AppConfig } from "$rs/AppConfig";
import { ConfigUpdate } from "$rs/ConfigUpdate";
import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import { createResource } from "solid-js";

const [appConfig, { mutate }] = createResource<AppConfig>(async () => await invoke("get_config"), {
    initialValue: {
        history_amount: 0,
        cache_dir: "",
        cache_size: 0,
        sources: [],
        display_background: false,
        interval: {
            nanos: 0,
            secs: 10
        },
        theme: "default",
        plugin_host_mode: "Daemon",
        plugins_dir: null,
    }
});
export { appConfig };
export const updateConfig = async (update: ConfigUpdate) => {
    await invoke("update_config", { update });
}
listen("config_changed", ({ payload }: { payload: AppConfig }) => {
    mutate(payload);
});

// const configContext = createContext([undefined as any, () => {}] as ReturnType<typeof createStore<AppConfig>>);

// export const ConfigProvider = lazy(async () => {
//     const initConfig: AppConfig = await invoke("get_config");
//     return {
//         default: (props: { children: JSX.Element }) => {
//             const c = children(() => props.children);
//             const [config, updateConfig] = createStore(initConfig);
//             listen("config_changed", ({ payload }) => updateConfig(payload as AppConfig));

//             return <configContext.Provider value={[config, updateConfig]} >{c()}</configContext.Provider>
//         }
//     };
// });

// export const useConfig = () => useContext(configContext);