import { invoke } from "@tauri-apps/api";
import { createResource } from "solid-js";
import { Config } from "../src-tauri/bindings/Config";

const ConfigComponent = () => {
    const [config, { refetch }] = createResource<Config>(() => invoke("get_config"));
    return <div>
        {config() && <div>
            {//checkbox
            }
            <label><input type="checkbox" checked={!!config()?.allow_nsfw} onChange={async event => {
                await invoke("set_config", { config: { showNsfw: event.currentTarget.checked } });
                refetch();
            }} />Allow NSFW</label>
        </div>}
        Config
    </div>
}

export default ConfigComponent;