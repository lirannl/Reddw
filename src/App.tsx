import { invoke } from "@tauri-apps/api";
import { createResource } from "solid-js";
import Config from "./Config";
import Sources from "./Sources";

function App() {
  const [path] = createResource<string>(() => invoke("config_dir"))
  return <>
    {path()}
    {/* <Config />
    <Sources /> */}
  </>
}

export default App;
