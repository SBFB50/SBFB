// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase H — intra-line word diff (fold V1). This is a MINI-diff of
// TWO already-paired lines (an old `del` content vs its `add` replacement),
// NOT a re-diff of the file: the file-level hunks come from Rust
// (`parse_unified_diff`, the single source of truth). It only refines the
// highlight WITHIN a changed line so the eye lands on the edited tokens.
//
// Token-level LCS (words / whitespace runs / single punctuation), 0 runtime
// dependency (Day-0 D2: Base UI is the only runtime dep — no jsdiff). Bounded:
// past `MAX_TOKENS` per side (a minified / vendored line) it degrades to a
// single "whole line changed" segment so the O(n·m) table never blows up.

export interface WordSeg {
  text: string
  changed: boolean
}

export interface WordDiff {
  old: WordSeg[]
  new: WordSeg[]
}

const MAX_TOKENS = 240

/** Split into identifiers, whitespace runs, and single punctuation chars so
 * `let ok = false;` → ['let',' ','ok',' ','=',' ','false',';']. */
function tokenize(s: string): string[] {
  return s.match(/[A-Za-z0-9_]+|\s+|[^A-Za-z0-9_\s]/g) ?? []
}

/** Merge adjacent tokens carrying the same changed flag into one segment. */
function coalesce(tokens: string[], changed: boolean[]): WordSeg[] {
  const segs: WordSeg[] = []
  for (let k = 0; k < tokens.length; k++) {
    const last = segs[segs.length - 1]
    if (last && last.changed === changed[k]) last.text += tokens[k]
    else segs.push({ text: tokens[k], changed: changed[k] })
  }
  return segs
}

/**
 * Highlight the tokens that differ between an old line and its replacement.
 * Common tokens (the LCS) are `changed:false`; the rest are `changed:true`.
 */
export function wordDiff(oldText: string, newText: string): WordDiff {
  const a = tokenize(oldText)
  const b = tokenize(newText)

  if (a.length > MAX_TOKENS || b.length > MAX_TOKENS) {
    return {
      old: oldText ? [{ text: oldText, changed: true }] : [],
      new: newText ? [{ text: newText, changed: true }] : [],
    }
  }

  const n = a.length
  const m = b.length
  // dp[i][j] = LCS length of a[i:] and b[j:].
  const dp: number[][] = Array.from({ length: n + 1 }, () => new Array<number>(m + 1).fill(0))
  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      dp[i][j] = a[i] === b[j] ? dp[i + 1][j + 1] + 1 : Math.max(dp[i + 1][j], dp[i][j + 1])
    }
  }

  const oldChanged = new Array<boolean>(n).fill(true)
  const newChanged = new Array<boolean>(m).fill(true)
  let i = 0
  let j = 0
  while (i < n && j < m) {
    if (a[i] === b[j]) {
      oldChanged[i] = false
      newChanged[j] = false
      i++
      j++
    } else if (dp[i + 1][j] >= dp[i][j + 1]) {
      i++
    } else {
      j++
    }
  }

  return { old: coalesce(a, oldChanged), new: coalesce(b, newChanged) }
}
