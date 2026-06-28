// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase E — signature (1) "token settle": a live counter posts its
// new value with a small upward settle, so a change READS as a change. Meaning,
// not decor: the settle is the value arriving.
//
// CSS-ONLY (a transform keyframe in index.css, `.motion-settle`), so it carries
// ZERO Motion-lib weight — this component renders in the EAGER hero (the
// orientation bar) and must not pull `m.*` into the `index` chunk. Re-keying on
// `value` remounts the span, which replays the keyframe. The keyframe is
// transform-only (translateY) so it collapses to its final frame under the
// `@media (prefers-reduced-motion)` belt-and-braces rule. `tabular-nums` keeps
// the digit box from reflowing as the number changes.
import { cn } from '../../lib/cn'

export function TokenCount({ value, className }: { value: string | number; className?: string }) {
  return (
    <span key={String(value)} data-testid="token-count" className={cn('motion-settle tabular-nums', className)}>
      {value}
    </span>
  )
}
