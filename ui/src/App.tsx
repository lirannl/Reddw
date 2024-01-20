import { invoke } from "@tauri-apps/api";
import Config from "./config/Config";
import Log from "./Log";
import { createEffect, on } from "solid-js";
import { appConfig } from "./context/config";

function App() {
  // Theme control
  createEffect(on(appConfig, config => {
    document.documentElement.dataset.theme = config.theme;
  }));

return (
  <>
    <Config />
    <Log />
    <div class="card">
      <div class="card-body">
        <button class="btn btn-primary" onClick={() => invoke("update_wallpaper")}>Update wallpaper</button>
        <button class="btn btn-warning" onClick={() => invoke("exit")}>Quit</button>
      </div>
    </div>
  </>
)
};

export default App;
