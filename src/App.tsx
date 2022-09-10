import { invoke } from "@tauri-apps/api";
import { createResource } from "solid-js";
// import Config from "./Config";
import Sources from "./Sources";

function App() {
  return <>
    {/* <Config /> */}
    <Sources />
  </>
}

export default App;
