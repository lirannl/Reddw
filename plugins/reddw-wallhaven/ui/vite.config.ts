import { defineConfig } from 'vite'
import solid from 'vite-plugin-solid'
//@ts-ignore
import path from "path"

export default defineConfig({
  plugins: [solid()],
  build: {
    lib: {
      //@ts-ignore
      entry: path.resolve(__dirname, 'src/webComponent.ts'),
      name: 'my-component',
      fileName: (format) => `my-component.${format}.js`,
      formats: ["es"],
    },
  }
})
