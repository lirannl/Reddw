import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  define: {
    __PROJECT_PATH__: process.env.NODE_ENV === "development" ? `"${btoa(process.cwd())}"` : "undefined",
  },
})
