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
  },
  build: {
    rolldownOptions: {
      output: {
        manualChunks(id) {
          // React core
          if (id.includes('node_modules/react/') || id.includes('node_modules/react-dom/') || id.includes('node_modules/react-router-dom/')) {
            return 'vendor-react';
          }
          // Graph libraries (G6, sigma, graphology, reagraph, force-graph)
          if (id.includes('node_modules/@antv/') || id.includes('node_modules/sigma/') || id.includes('node_modules/graphology') || id.includes('node_modules/reagraph/') || id.includes('node_modules/@react-sigma/') || id.includes('node_modules/react-force-graph')) {
            return 'vendor-graph';
          }
          // Chart libraries (recharts, nivo)
          if (id.includes('node_modules/recharts/') || id.includes('node_modules/@nivo/')) {
            return 'vendor-charts';
          }
          // Map libraries (leaflet)
          if (id.includes('node_modules/leaflet/') || id.includes('node_modules/react-leaflet/')) {
            return 'vendor-map';
          }
          // UI primitives (radix)
          if (id.includes('node_modules/@radix-ui/')) {
            return 'vendor-ui';
          }
        },
      },
    },
  },
})
