import { resolve } from 'node:path'
import { defineConfig, externalizeDepsPlugin } from 'electron-vite'
import vue from '@vitejs/plugin-vue'

const root = import.meta.dirname
const mode = process.env.NODE_ENV || 'production'

export default defineConfig({
  main: {
    plugins: [externalizeDepsPlugin()],
    build: {
      rollupOptions: {
        input: { index: resolve(root, 'src/main/index.js') }
      }
    }
  },
  preload: {
    plugins: [externalizeDepsPlugin()],
    build: {
      rollupOptions: {
        input: { index: resolve(root, 'src/preload/index.js') }
      }
    }
  },
  renderer: {
    root: resolve(root, 'src/renderer'),
    resolve: {
      // The codebase imports .vue components without the extension (as webpack did).
      extensions: ['.mjs', '.js', '.json', '.vue'],
      alias: {
        '@': resolve(root, 'src/renderer')
      }
    },
    // The renderer no longer runs Node, but a few files still read build-time
    // flags via `process.env.*`. Vite replaces them at build time.
    define: {
      'process.env.IS_WEB': 'false',
      'process.env.WEB_DEMO_MODE': 'false',
      'process.env.NODE_ENV': JSON.stringify(mode),
      'process.env.ENV': JSON.stringify(mode),
      'process.env.BUILD': JSON.stringify('arm64')
    },
    build: {
      rollupOptions: {
        input: { index: resolve(root, 'src/renderer/index.html') }
      }
    },
    plugins: [vue()]
  }
})
