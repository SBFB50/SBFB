// SPDX-License-Identifier: AGPL-3.0-or-later
import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import type { CommitDiff } from '../../api/operator'
import { DiffView } from './DiffView'

// DiffView is pure: it renders the hunks computed IN RUST (parse_unified_diff),
// never a JS re-diff. The line numbers come from the backend DiffLine, and each
// kind (add/del/ctx) gets its tone/prefix. This guards that mapping (review P2).
const DIFF: CommitDiff = {
  sha: 'a5ace8d0',
  title: 'auth cookie',
  files: [
    {
      path: 'crates/sbfb-factory/src/auth.rs',
      insertions: 1,
      deletions: 1,
      hunks: [
        {
          header: '@@ -10,3 +10,3 @@ fn auth',
          lines: [
            { kind: 'ctx', content: 'let token = header;', old_lineno: 10, new_lineno: 10 },
            { kind: 'del', content: 'let ok = false;', old_lineno: 11, new_lineno: null },
            { kind: 'add', content: 'let ok = true;', old_lineno: null, new_lineno: 11 },
            { kind: 'ctx', content: 'return ok;', old_lineno: 12, new_lineno: 12 },
          ],
        },
      ],
    },
  ],
}

describe('DiffView (J11 — renders Rust-computed hunks, never a JS re-diff)', () => {
  it('renders the file header with the backend insertion/deletion counts', () => {
    render(<DiffView diff={DIFF} />)
    expect(screen.getByTestId('diff-view')).toBeInTheDocument()
    expect(screen.getByText('crates/sbfb-factory/src/auth.rs')).toBeInTheDocument()
    expect(screen.getByText('+1')).toBeInTheDocument() // insertion count from the backend
    expect(screen.getByText('@@ -10,3 +10,3 @@ fn auth')).toBeInTheDocument()
  })

  it('renders each diff line verbatim with its backend line numbers', () => {
    render(<DiffView diff={DIFF} />)
    // The inline code is preserved verbatim (the +/-/space marker is the gutter,
    // not part of the content) — the content came straight from the Rust hunk.
    expect(screen.getByText('let ok = false;')).toBeInTheDocument()
    expect(screen.getByText('let ok = true;')).toBeInTheDocument()
    // Backend line numbers are restituted (a del has only an old lineno, an add
    // only a new one) — there are two cells showing "11" (old of del, new of add).
    expect(screen.getAllByText('11')).toHaveLength(2)
    expect(screen.getAllByText('10')).toHaveLength(2) // ctx: old + new
  })

  it('shows an honest empty state for a diff with no files', () => {
    render(<DiffView diff={{ sha: 'deadbeef', title: 'empty', files: [] }} />)
    expect(screen.getByText(/aucun fichier dans ce diff/)).toBeInTheDocument()
  })
})
