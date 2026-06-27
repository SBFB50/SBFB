// SPDX-License-Identifier: AGPL-3.0-or-later
import { clsx, type ClassValue } from 'clsx'

// Sprint 80 Phase C: clsx-only. The greenfield front composes class lists
// conditionally but never relies on Tailwind conflict resolution (no
// component takes an external className that overrides an internal utility),
// so `tailwind-merge` was dead weight (~22 kB in the hero app chunk) and is
// dropped. Re-introduce it (with a justified size-limit bump) the day a
// primitive genuinely needs to merge a caller-supplied className.
export function cn(...inputs: ClassValue[]): string {
  return clsx(inputs)
}
