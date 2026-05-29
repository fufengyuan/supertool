import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'
import { resolve } from 'node:path'
import pkg from './package.json'

const host = process.env.TAURI_DEV_HOST

// 去掉 crossorigin 属性 — Tauri 自定义协议不支持 CORS
const removeCrossorigin = () => ({
  name: 'remove-crossorigin',
  transformIndexHtml(html: string) {
    return html.replace(/\s*crossorigin\s*/g, ' ')
  },
})

export default defineConfig({
  plugins: [
    vue(),
    tailwindcss(),
    removeCrossorigin(),
  ],
  base: './',
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
    __VUE_I18N_LEGACY_API__: JSON.stringify(true),
    __VUE_I18N_FULL_INSTALL__: JSON.stringify(true),
    __INTLIFY_PROD_DEVTOOLS__: JSON.stringify(false),
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: 'ws', host, port: 1421 }
      : undefined,
    watch: {
      ignored: ['**/tauri/**', '**/target/**', '**/.git/**'],
    },
  },
  test: {
    globals: true,
    environment: 'happy-dom',
  },
})
