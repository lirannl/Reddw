import './app.css'
import setupLocatorUI from "@locator/runtime";
import Main from './Main.svelte'

if (import.meta.env.DEV) {
  setupLocatorUI({
    adapter: "svelte",
    projectPath: `${atob(__PROJECT_PATH__)}/`,
  });
}

const app = new Main({
  target: document.body,
})

export default app
