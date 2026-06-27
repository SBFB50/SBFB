// SPDX-License-Identifier: AGPL-3.0-or-later
import { cn } from './lib/cn'

// Sprint 80 Phase B — minimal greenfield shell. The bi-focal STEER /
// VERIFY surfaces and the ambient rail land in Phases C→H; this scaffold
// proves the oklch @theme utilities generate (ADAPT-2), the dark theme
// applies, Geist sans/mono are vendored, and the build is CSP-clean
// (asserted by e2e/boot.spec.ts on the BUILT bundle). No inline styles
// (CSP `default-src 'self'`): every surface is a Tailwind utility.
export function App() {
  return (
    <div className="min-h-screen bg-s0 text-tx font-sans">
      {/* Placeholder for the altitude-0 ambient rail (wired Phase C). */}
      <header
        data-testid="operator-rail"
        className="flex items-center gap-3 border-b border-bd bg-s1 px-4 py-2"
      >
        <span className="font-mono text-tx tabular-nums">Factory Operator</span>
        <span className="text-tx3" aria-hidden>
          ·
        </span>
        <span className="text-sm text-tx2">établi bi-focal — scaffold S80 Phase B</span>
      </header>
      <main className="p-6">
        <section className={cn('max-w-prose rounded-md border border-bd bg-s1 p-4')}>
          <h1 className="text-lg text-tx">Fondations en place</h1>
          <p className="mt-2 text-tx2">
            React&nbsp;19, Tailwind&nbsp;v4 (tokens oklch), Base&nbsp;UI, Motion et
            Geist sont câblés. Les focales STEER et VERIFY arrivent aux phases
            suivantes du sprint.
          </p>
        </section>
      </main>
    </div>
  )
}
