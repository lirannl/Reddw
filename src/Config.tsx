import { invoke } from "@tauri-apps/api";
import { createEffect, createResource } from "solid-js";
import { createForm } from "./utils/Form";
import { match, P } from "ts-pattern";

const ConfigComponent = () => {
    const [config, { refetch }] = createResource<Config>(() => invoke("get_config"));
    const { Form, fieldProps, state, setFormState } = createForm(
        (e: Config) => ({ allow_nsfw: (e.allow_nsfw ? "on" as const : undefined) }),
        f => ({ allow_nsfw: f.allow_nsfw === "on" ? 1 : 0 }),
        config
    );

    createEffect(() => {
        const currentState = state();
        match(currentState)
            .with({
                entity: P.not(undefined),
            }, ({ form }) => form.allow_nsfw === "on")
            .run();

    });

    return <>
        <button onClick={() => {
            setFormState({ allow_nsfw: 1 });
        }}>External update</button>
        <Form>
            <label for="allow_nsfw">Allow NSFW</label>
            <input type="checkbox" title="Allow NSFW"
                {...fieldProps("allow_nsfw", v => ({ checked: !!v }))} />
        </Form>
    </>
}

export default ConfigComponent;