import { invoke } from "@tauri-apps/api";
import { createEffect, createResource } from "solid-js";
import { createForm } from "./utils/Form";

const ConfigComponent = () => {
    const [config, { refetch }] = createResource<Config>(() => invoke("get_config"));
    const { Form, state, fieldProps } = createForm(
        (e: Config) => ({ allow_nsfw: e.allow_nsfw ? "on" : "" }),
        f => ({ allow_nsfw: f.allow_nsfw === "on" ? 1 : 0 }),
        config());
    createEffect(() => { console.log(state()) })
    return <>
        <Form>
            <input type="checkbox" title="Allow NSFW" {...fieldProps("allow_nsfw")} />
            <button onClick={() => {
                invoke("set_config", state()).then(refetch);
            }}>Save</button>
        </Form>
    </>
}

export default ConfigComponent;