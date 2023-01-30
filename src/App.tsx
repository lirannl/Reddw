import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import { Component, createSignal } from 'solid-js';
import { Config } from "./Config";

const App: Component = () => {
  const [loading, setLoading] = createSignal(false);
  listen("update_wallpaper_start", () => setLoading(true));
  listen("update_wallpaper_stop", () => setLoading(false));
  return <>
    <div><Config /></div>
    {loading() && <p>Loading...</p>}
    <button onClick={async () => invoke("update_wallpaper")}>Manual update</button>
    <button onClick={async () => invoke("cache_queue")}>Cache queue</button>
    <button onClick={async () => invoke("get_queue").then(console.log)}>Get queue</button>
  </>;
};

export default App;
