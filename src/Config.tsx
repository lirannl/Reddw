import { invoke } from "@tauri-apps/api";
import { createEffect, createResource } from "solid-js";
import { createForm } from "./utils/Form";

const ConfigComponent = () => {
    const [config, { refetch, mutate }] = createResource<Config>(() => invoke("get_config"));
    const { fieldProps } = createForm({ value: config() });
    createEffect(() => { console.log(config()) })
    return <form onChange={e => {
        const formData = {} as unknown as { allow_nsfw?: "on" };
        new FormData(e.currentTarget).forEach((value, key) => {
            //@ts-ignore
            formData[key] = value;
        });
        mutate({
            allow_nsfw: formData.allow_nsfw === "on" ? 1 : 0,
        })
    }}>
        <input type="checkbox" {...fieldProps("allow_nsfw")} />
    </form>
}

export default ConfigComponent;