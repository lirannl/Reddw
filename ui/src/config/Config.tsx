import { debounce } from "@solid-primitives/scheduled"
import { appConfig, updateAppConfig } from "../context/config"
import Sources from "./Sources"
import { AppConfig } from "$rs/AppConfig"
import Range from "../components/Range";
import { log } from "../Log";
import { invoke } from "@tauri-apps/api";

function update<Prop extends keyof AppConfig, EventValue, Transformer extends AppConfig[Prop] extends EventValue ? undefined : (v: EventValue) => AppConfig[Prop]>(prop: Prop, transformer?: Transformer) {
    return debounce((event: { target: { value: EventValue; }; }) => {
        updateAppConfig({ ...appConfig(), [prop]: transformer ? transformer(event.target.value) : event.target.value });
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
                    <input type="checkbox" class="checkbox mx-2 join-item" checked={appConfig().display_background} onInput={e => {
                        updateAppConfig({ ...appConfig(), display_background: e.target.checked });
                    }} />
                </label>
            </div>
            <div class="join">
                <label class="join-item">
                    Cache directory
                    <input class="join-item input" value={appConfig().plugins_dir ?? undefined} onInput={update("plugins_dir")} />
                    <button class="join-item btn btn-primary" onClick={async () => {
                        log(await invoke("select_folder"), "Info");
                    }}>Select file</button>
                </label>
            </div>
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