import { invoke } from "@tauri-apps/api";
import { createResource, createSignal, createEffect, For } from "solid-js";
import { TransitionGroup } from "solid-transition-group";
import { createForm } from "./utils/Form";

const Sources = () => {
    const [sources, { refetch }] = createResource<Source[]>(() => invoke('get_sources'));
    const [error, setError] = createSignal<string>();
    const { Form, fieldProps } = createForm<Source>({
        onSubmit: async (source) => {
            try {
                await invoke('add_source', { source });
                refetch();
            } catch (e: any) {
                setError(e);
            }
        }
    })

    // Make errors last 1000ms
    createEffect(() => {
        if (typeof error() === "string") {
            setTimeout(setError, 1000);
        }
    })

    return (
        <div>
            {sources() && <TransitionGroup name="fade">
                <For each={sources()}>
                    {source => <div>{source.subreddit}<button onClick={() => invoke("remove_source", { id: source.id })
                        .then(refetch)}>-</button></div>}
                </For>
            </TransitionGroup>}
            <Form>
                <input {...fieldProps("subreddit")} />
                <button type="submit">+</button>
                {typeof error() !== "undefined" && <span class="error">{error()}</span>}
            </Form>
        </div>
    );
}

export default Sources;