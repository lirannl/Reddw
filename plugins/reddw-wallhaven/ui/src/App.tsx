import { createSignal } from "solid-js";
import Component, { PluginConfig } from "./Component";

export default () => {
  const [config, update_config] = createSignal<PluginConfig>({ tags: [] })
  return <>
    <button onClick={() => {
      update_config({ tags: ["default", "value"] });
    }}>Change</button>
    <Component value={config()} />
  </>
}