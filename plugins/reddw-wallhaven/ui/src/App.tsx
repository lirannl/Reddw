import { createSignal } from "solid-js";
import Component, { PluginConfig } from "./Component";

export default () => {
  const [config, update_config] = createSignal<PluginConfig>({ searchTerms: [] })
  return <>
    <button onClick={() => {
      update_config({ searchTerms: ["External update"] });
    }}>Change</button>
    <Component value={config()} />
  </>
}