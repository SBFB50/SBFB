import path from 'path'
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import { visualizer } from 'rollup-plugin-visualizer'

// Sprint 9 Phase A (D6) — code splitting.
//
// The manualChunks routine is guarded on `node_modules` first
// (lobe-chat's 2025 pattern) because any user-land id must fall
// through to rolldown's per-entry auto-chunking — if we try to
// assign a user-land module to a named chunk while it's imported
// by a lazy route, rolldown conservatively hoists it into the
// main chunk and the code split evaporates. The guard keeps our
// named vendor chunks and lets the lazy routes land in the
// per-page chunks that the React Router `lazy: () => import()`
// calls create implicitly.
//
// The two feature chunks (`tabview`, `palette`) are src-prefixed
// and therefore bypass the node_modules guard — they live above
// it because they're the first two src-side chunks we split out.
//
// Dead chunks retired (Sprint 5 deleted the underlying deps but
// the chunk bucket survived):
//   - `vendor-graph`  (reagraph / sigma / graphology / @antv / react-force-graph)
//   - `vendor-charts` (recharts / @nivo)
//   - `vendor-map`    (leaflet / react-leaflet)
// See `docs/shell/PATTERNS.md` P12 for the full rationale.

const ANALYZE = process.env.ANALYZE_MODE === 'true'

export default defineConfig({
  plugins: [
    react(),
    tailwindcss(),
    // Opt-in bundle visualization. Run
    //   ANALYZE_MODE=true npm run build
    // to emit `dist/stats.html` with a treemap of the bundle.
    // Kept out of the default build to avoid shipping ~1 MB of
    // HTML + JSON into a regular `dist/`.
    ANALYZE &&
      visualizer({
        filename: 'dist/stats.html',
        open: false,
        gzipSize: true,
        brotliSize: true,
      }),
  ],
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
          // Normalize Windows backslashes so path tests below
          // work the same on every host.
          const nid = id.replace(/\\/g, '/')

          // No manual src-side chunks. Sprint 9 Phase A learned
          // the hard way that naming a manual chunk over a
          // user-land directory makes rolldown anchor shared
          // dependencies (react, react-dom, @base-ui) into that
          // chunk and turn the would-be vendor chunks into
          // dependents — total bytes go up, eager paint suffers.
          // Instead we keep src files unassigned so rolldown
          // creates per-lazy-route chunks naturally:
          //   - `Projects.tsx`         → assets/Projects-*.js
          //   - `command-palette/`     → assets/CommandPalette-*.js
          //     (anchored by `React.lazy()` in AppShell)
          //   - `app/tabview/`         → folded into ProjectDetail
          //     and AppTabPage chunks (small enough to live
          //     inline; the duplication is a few KB per consumer
          //     and avoids the vendor-anchor pathology).
          //
          // Only split out named vendor chunks for library code.
          if (!nid.includes('/node_modules/')) return

          // React core (router v6 ships as react-router-dom).
          if (
            nid.includes('/node_modules/react/') ||
            nid.includes('/node_modules/react-dom/') ||
            nid.includes('/node_modules/react-router-dom/') ||
            nid.includes('/node_modules/react-router/') ||
            nid.includes('/node_modules/scheduler/')
          ) {
            return 'vendor-react'
          }

          // State + data-fetching + schema validation layer.
          // Zod is the schema validator we use to parse every
          // coordinator response, so it's load-bearing on the
          // same paint as @tanstack/react-query — pinning it to
          // the same chunk avoids a 30 KB rolldown common chunk
          // that would name itself after whichever module
          // imported zod first (`projectStore-*.js` in our case).
          if (
            nid.includes('/node_modules/@tanstack/') ||
            nid.includes('/node_modules/zustand/') ||
            nid.includes('/node_modules/zod/')
          ) {
            return 'vendor-query'
          }

          // Headless UI primitives: @base-ui/react ships the
          // accessibility layer our shadcn-style components wrap
          // (Dialog, Tooltip, Tabs, ScrollArea, ...). @radix-ui/*
          // packages are listed in package.json as leftover
          // dependencies from the legacy UI — we keep them in the
          // same bucket so anything that still imports them lands
          // here rather than in a feature chunk. `cmdk` is the
          // palette's lone non-Base UI primitive; we bundle it
          // with the rest of the UI vendor code so the palette
          // chunk stays small.
          //
          // The classname utilities (`clsx`, `tailwind-merge`,
          // `class-variance-authority`) are pulled by every UI
          // primitive transitively, so we anchor them to the same
          // bucket. Without an explicit assignment rolldown
          // extracts them into a misnamed common chunk like
          // `projectStore-*.js` (94 KB of `tailwind-merge`'s
          // class group config travels under whichever module it
          // first encountered).
          if (
            nid.includes('/node_modules/@base-ui/') ||
            nid.includes('/node_modules/@radix-ui/') ||
            nid.includes('/node_modules/cmdk/') ||
            nid.includes('/node_modules/tailwind-merge/') ||
            nid.includes('/node_modules/clsx/') ||
            nid.includes('/node_modules/class-variance-authority/')
          ) {
            return 'vendor-ui'
          }
        },
      },
    },
  },
})
