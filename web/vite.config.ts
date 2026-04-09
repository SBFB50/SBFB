import path from 'path'
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    proxy: {
      '/api': 'http://localhost:8000',
      '/ollama': {
        target: 'http://localhost:11434',
        rewrite: (path) => path.replace(/^\/ollama/, ''),
        changeOrigin: true,
      },
    },
  }
})
