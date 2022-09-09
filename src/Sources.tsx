import { invoke } from "@tauri-apps/api";
import { createResource, createSignal, createEffect, For } from "solid-js";
import { TransitionGroup } from "solid-transition-group";

const Sources = () => {
    const [sources, { refetch }] = createResource<{ subreddit: string, id: number }[]>(() => invoke('get_sources'));
    const [error, setError] = createSignal<string>();

    // Make errors last 1000ms
    createEffect(() => {
        if (typeof error() === "string") {
            setTimeout(setError, 1000);
        }
    })
    return (
        <div>
            {sources() && <TransitionGroup name="slide">
                <For each={sources()}>
                    {source => <div>{source.subreddit}<button onClick={() => invoke("delete_source", { doomedId: source.id })
                        .then(refetch)}>-</button></div>}
                </For>
            </TransitionGroup>}
            <form onSubmit={async event => {
                event.preventDefault();
                const source = { subreddit: event.currentTarget["subreddit"].value }
                event.currentTarget["subreddit"].value = "";
                invoke("add_source", { source }).then(refetch).catch(setError);
            }}>
                <input placeholder="Subreddit" name="subreddit" />
                <button type="submit">+</button>
                {typeof error() !== "undefined" && <span class="error">{error()}</span>}
            </form>
        </div>
    );
}

export default Sources;