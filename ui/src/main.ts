import './app.css'
import setupLocatorUI from "@locator/runtime";
import App from './App.svelte'

if (import.meta.env.DEV) {
  setupLocatorUI({
    adapter: "svelte",
    projectPath: `${atob(__PROJECT_PATH__)}/`,
  });
}

const app = new App({
  target: document.body,
})

export default app
