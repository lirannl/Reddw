import { invoke } from "@tauri-apps/api";
import Config from "./Config";
import Log from "./Log";

function App() {
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
