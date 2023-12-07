import { AppConfig } from "$rs/AppConfig";
import { createStore } from "solid-js/store";
import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import { createSignal } from "solid-js";

export const [appConfig, updateAppConfig] = createSignal(await invoke("get_config") as AppConfig);
listen("config_changed", ({ payload }) => updateAppConfig(payload as AppConfig));

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