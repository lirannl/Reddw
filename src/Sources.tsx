import { invoke } from "@tauri-apps/api";
import { createResource, createSignal, createEffect, For } from "solid-js";
import { TransitionGroup } from "solid-transition-group";

const Sources = () => {
    const [sources, { refetch }] = createResource<Source[]>(() => invoke('get_sources'));
    const [error, setError] = createSignal<string>();

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
            <form onSubmit={e => {
                e.preventDefault();
                const formData = Object.fromEntries(new FormData(e.currentTarget).entries());
                invoke("add_source", { source: formData }).then(refetch).catch(setError);
            }}>
                <input placeholder="Subreddit" name="subreddit" />
                <button type="submit">+</button>
            </form>
            {typeof error() !== "undefined" && <span class="error">{error()}</span>}
        </div>
    );
}

export default Sources;