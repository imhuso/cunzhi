import process from 'node:process'
import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [
    vue(),
    tailwindcss(),
  ],
  clearScreen: false,
  // Tauri应用需要使用相对路径
  base: './',
  server: {
    port: 5176,
    strictPort: true,
    host: '0.0.0.0',
    hmr: {
      port: 5177,
    },
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: process.env.TAURI_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    chunkSizeWarningLimit: 1500,
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['vue', '@vueuse/core'],
          antd: ['ant-design-vue'],
          markdown: ['@f3ve/vue-markdown-it', 'markdown-it']
        }
      }
    }
  },
})
