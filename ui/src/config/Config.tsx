import { debounce } from "@solid-primitives/scheduled"
import { appConfig, updateConfig } from "../context/config"
import Sources from "./Sources"
import { AppConfig } from "$rs/AppConfig"
import Range from "../components/Range";
// import LogBehaviour from "./LogBehaviour";
import { log } from "../Log";
import { invoke } from "@tauri-apps/api";
import { AiOutlineFolder } from "solid-icons/ai";
// import { For } from "solid-js";

const update = <Prop extends Exclude<keyof AppConfig, "interval" | "sources">, EventValue,
    Transformer extends ((v: EventValue) => AppConfig[Prop]) | undefined = undefined>
    (prop: Prop, transformer?: Transformer) => {
    return debounce((event: ({ target: { value: undefined extends Transformer ? AppConfig[Prop] : EventValue } }) | AppConfig[Prop]) => {
        let value: AppConfig[Prop];
        if (typeof event === "object" && event !== null && "target" in event) {
            if (transformer) value = transformer(event.target.value as EventValue);
            else value = event.target.value as AppConfig[Prop];
        }
        else value = event;
        updateConfig({ Other: { ...appConfig(), [prop]: value } });
    }, 500);
}
export default () => {
    return <div class="card">
        <form class="card-body" onSubmit={e => { e.preventDefault() }}>
            <Range label="Cache size" value={appConfig().cache_size} max={10000} min={10} unit="MB" onInput={update("cache_size")} />
            <div class="join">
                <label class="join-item">
                    Theme
                    <input class="input join-item" value={appConfig().theme} onInput={update("theme")} />
                </label>
            </div>
            <div class="join">
                <label class="join-item">
                    Display background
                    <input type="checkbox" class="checkbox mx-2 join-item"
                        checked={appConfig().display_background}
                        onInput={e => update("display_background")(e.target.checked)} />
                </label>
            </div>
            <div class="join">
                <label class="join-item">
                    Plugins directory
                    <input class="join-item input" value={appConfig().plugins_dir ?? undefined} onInput={update("plugins_dir")} />
                    <button class="join-item btn btn-primary" onClick={async () => {
                        log(await invoke("select_folder"), "Info");
                    }}><AiOutlineFolder /></button>
                </label>
            </div>
            <div class="join">
                <label class="join-item">
                    Cache directory
                    <input class="join-item input" value={appConfig().cache_dir ?? undefined} onInput={update("cache_dir")} />
                    <button class="join-item btn btn-primary" onClick={async () => {
                        log(await invoke("select_folder"), "Info");
                    }}><AiOutlineFolder /></button>
                </label>
            </div>
            <div class="join">
                <label class="join-item">
                    Alternative changer command
                    <input class="join-item input" value={appConfig().setter_command ?? ""}
                        onInput={update("setter_command", (cmd: string) => cmd || null)} />
                </label>
            </div>
            {/* <div class="join">
                <label class="join-item">
                    Logging behaviours
                    <For each={[...appConfig()?.logging ?? [], undefined]}>{behaviour =>
                        <LogBehaviour existing={behaviour} />
                    }</For>
                </label>
            </div> */}
            <div class="collapse bg-base-200">
                <input type="checkbox" />
                <div class="collapse-title text-xl font-medium">
                    Sources
                </div>
                <div class="collapse-content"><Sources /></div>
            </div>
        </form>
    </div>
}