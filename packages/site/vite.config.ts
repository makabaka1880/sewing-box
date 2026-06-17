import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueDevTools from 'vite-plugin-vue-devtools'
import wasm from 'vite-plugin-wasm'

// https://vite.dev/config/
export default defineConfig(({ command }) => ({
  // Use /<repo>/ base on GitHub Pages; BASE env var lets the workflow override.
  base: process.env.BASE || '/',
  plugins: [
    vue(),
    command === 'serve' && vueDevTools(),
    wasm(),
  ].filter(Boolean),
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    },
  },
}))
