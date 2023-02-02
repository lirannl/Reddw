import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import { Component, createEffect, createSignal, For } from 'solid-js';
import { Config } from "./Config";

const App: Component = () => {
  const [loading, setLoading] = createSignal(false);
  const [latestMessage, setLatestMessage] = createSignal("");
  const [log, setLog] = createSignal<string[]>([]);
  listen("update_wallpaper_start", () => setLoading(true));
  listen("update_wallpaper_stop", () => setLoading(false));
  listen<string>("print", (event) => setLatestMessage(event.payload))

  createEffect(() => {
    if (latestMessage()) { setLog(log => [...log, latestMessage()]); setLatestMessage(""); }
  });

  return <>
    <div><Config /></div>
    {loading() && <p>Loading...</p>}
    <button onClick={async () => invoke("update_wallpaper")}>Manual update</button>
    <button onClick={async () => invoke("cache_queue")}>Cache queue</button>
    <button onClick={async () => invoke("exit")}>Quit</button>
    {log() && <For each={log()}>{(line) => <p>{line}</p>}</For>}
  </>;
};

export default App;
