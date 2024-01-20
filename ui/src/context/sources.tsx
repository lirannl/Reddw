import { createResource } from "solid-js";
import { invoke } from "../overrides";
import { listen } from "@tauri-apps/api/event";
import { log } from "../Log";

const [sources, { mutate }] = createResource(() => invoke<string[]>("query_available_source_plugins"));

listen("source_added", ({ payload }: { payload: string }) => {
    mutate([...sources() ?? [], payload]);
    log(`Source added: "${payload}"`, "Info");
})
listen("source_removed", ({ payload }: { payload: string }) => {
    mutate(sources()?.filter(source => payload !== source));
    log(`Source removed: "${payload}"`, "Info");
})

export { sources };