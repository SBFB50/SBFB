// SPDX-License-Identifier: AGPL-3.0-or-later
import { render, screen, fireEvent, within } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import type { FileDiff } from '../../../api/operator'
import { DiffViewer } from './DiffViewer'

// DiffViewer renders the hunks computed IN RUST (parse_unified_diff), never a
// JS re-diff. The SAME component serves the working tree (/api/git/diff) and a
// past commit (/sprint-history/diff/{sha}) on the shared FileDiff[] shape (fold
// V2/U7). Word-diff only refines the highlight within a del→add replacement.
const FILES: FileDiff[] = [
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
]

describe('DiffViewer (bespoke VERIFY diff-viewer — folds V1/V2/V3)', () => {
  it('renders the Rust hunk header, context lines verbatim and backend line numbers', () => {
    render(<DiffViewer files={FILES} />)
    expect(screen.getByTestId('verify-diff')).toBeInTheDocument()
    expect(screen.getByText('@@ -10,3 +10,3 @@ fn auth')).toBeInTheDocument()
    expect(screen.getByText('let token = header;')).toBeInTheDocument()
    expect(screen.getByText('return ok;')).toBeInTheDocument()
    // del old-lineno 11 + add new-lineno 11 → exactly two "11" gutter cells.
    expect(screen.getAllByText('11')).toHaveLength(2)
  })

  it('highlights only the changed tokens of a del→add replacement (word-diff)', () => {
    render(<DiffViewer files={FILES} />)
    const changed = screen.getAllByTestId('word-changed').map((n) => n.textContent)
    expect(changed).toContain('false')
    expect(changed).toContain('true')
    expect(changed).not.toContain('let ok = ')
  })

  it('toggles to side-by-side — the context line then appears on BOTH sides', () => {
    render(<DiffViewer files={FILES} />)
    expect(screen.getAllByText('return ok;')).toHaveLength(1)
    fireEvent.click(screen.getByTestId('diff-mode-split'))
    expect(screen.getByTestId('diff-mode-split')).toHaveAttribute('aria-pressed', 'true')
    expect(screen.getAllByText('return ok;')).toHaveLength(2)
  })

  it('collapses the change-set column (→ diff plein)', () => {
    render(<DiffViewer files={FILES} />)
    expect(screen.getByTestId('diff-changeset')).toBeInTheDocument()
    fireEvent.click(screen.getByTestId('changeset-toggle'))
    expect(screen.queryByTestId('diff-changeset')).toBeNull()
  })

  it('routes a hunk correction as an INTENTION (never Approve/Merge/Commit)', () => {
    const onHunkIntent = vi.fn()
    render(<DiffViewer files={FILES} onHunkIntent={onHunkIntent} />)
    fireEvent.click(screen.getByTestId('hunk-intent'))
    expect(onHunkIntent).toHaveBeenCalledWith('crates/sbfb-factory/src/auth.rs', '@@ -10,3 +10,3 @@ fn auth')
  })

  it('renders a past commit through the SAME component (bi-usage V2/U7)', () => {
    render(<DiffViewer files={FILES} caption="commit a5ace8d0 — auth" testid="diff-view" />)
    expect(screen.getByTestId('diff-view')).toBeInTheDocument()
    expect(screen.getByText(/commit a5ace8d0/)).toBeInTheDocument()
  })

  it('shows an honest empty state for a clean tree', () => {
    render(<DiffViewer files={[]} emptyLabel="arbre de travail propre · rien à examiner" />)
    expect(screen.getByText('arbre de travail propre · rien à examiner')).toBeInTheDocument()
  })

  it('flags a backend-truncated diff (fold V3 ◦ tronqué)', () => {
    render(<DiffViewer files={FILES} truncated />)
    expect(screen.getByText(/tronqué/)).toBeInTheDocument()
  })

  it('renders a density minimap with one bar per file (fold V3)', () => {
    const two: FileDiff[] = [FILES[0], { ...FILES[0], path: 'b.rs' }]
    render(<DiffViewer files={two} />)
    const minimap = screen.getByTestId('diff-minimap')
    expect(within(minimap).getAllByRole('button')).toHaveLength(2)
  })

  it('navigates hunks with the keyboard — aria-current moves (fold V3)', () => {
    const two: FileDiff[] = [FILES[0], { ...FILES[0], path: 'b.rs' }]
    render(<DiffViewer files={two} />)
    const hunks = screen.getAllByTestId('diff-hunk')
    expect(hunks[0]).toHaveAttribute('aria-current', 'true')
    fireEvent.keyDown(screen.getByTestId('diff-scroll'), { key: 'ArrowDown' })
    expect(screen.getAllByTestId('diff-hunk')[1]).toHaveAttribute('aria-current', 'true')
  })

  it('keeps the word-diff highlight in side-by-side mode (fold V1)', () => {
    render(<DiffViewer files={FILES} />)
    fireEvent.click(screen.getByTestId('diff-mode-split'))
    const changed = screen.getAllByTestId('word-changed').map((n) => n.textContent)
    expect(changed).toContain('false')
    expect(changed).toContain('true')
  })

  it('renders a hostile diff line as INERT text (XSS regression guard)', () => {
    const hostile: FileDiff[] = [
      {
        path: 'evil.ts',
        insertions: 1,
        deletions: 0,
        hunks: [
          {
            header: '@@ -1 +1 @@',
            lines: [{ kind: 'add', content: '<script>alert(1)</script>', old_lineno: null, new_lineno: 1 }],
          },
        ],
      },
    ]
    const { container } = render(<DiffViewer files={hostile} />)
    // shown as literal text, never parsed into a live <script> element.
    expect(screen.getByText('<script>alert(1)</script>')).toBeInTheDocument()
    expect(container.querySelector('script')).toBeNull()
  })
})
