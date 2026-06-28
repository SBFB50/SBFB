// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase H — the bespoke VERIFY diff-viewer (folds V1/V2/V3). It
// renders the hunks computed IN RUST (`parse_unified_diff`, the single source
// of truth — NEVER a JS re-diff; kickoff invariant #11), on the SAME
// `FileDiff[]` shape served by BOTH `/api/git/diff` (working tree) and
// `/api/sprint-history/diff/{sha}` (a past commit) — so ONE component serves
// the two usages (fold V2/U7). 0 runtime dependency (Day-0 D2: Base UI only —
// no react-diff-view, no jsdiff, no @tanstack/virtual): bi-mode (inline ⇄
// side-by-side), intra-line word-diff (the in-house token LCS of two paired
// lines, ./wordDiff), keyboard hunk navigation + a density minimap, and a
// collapsible change-set column. Hunk actions are INTENTIONS routed to the
// session ("transmettre la correction"), never Approve/Merge/Commit.
//
// This module lives under verify/plein/ so the vite manualChunk pulls it into
// the dedicated `diff-viewer` async chunk (measured by .size-limit.json),
// keeping the VerifyScene hero chunk under its budget (preflight §4).
import { useEffect, useMemo, useRef, useState, type ReactNode } from 'react'
import type { DiffLine, FileDiff } from '../../../api/operator'
import { wordDiff, type WordSeg } from './wordDiff'

export type DiffMode = 'inline' | 'split'

export interface DiffViewerProps {
  files: FileDiff[]
  /** A source label (e.g. "working-tree" or a commit sha/title). */
  caption?: ReactNode
  /** Honest empty state when there is nothing to show. */
  emptyLabel?: string
  /** The backend truncated the diff at MAX_DIFF_LINES (working tree). */
  truncated?: boolean
  /** Route a hunk correction to the session as an intention (never execute). */
  onHunkIntent?: (file: string, hunkHeader: string) => void
  testid?: string
}

interface PreparedLine {
  line: DiffLine
  /** Word-diff segments when this line is half of a del→add replacement. */
  words?: WordSeg[]
}
interface PreparedHunk {
  header: string
  lines: PreparedLine[]
  /** Global hunk id (across all files) for keyboard navigation. */
  gid: number
}
interface PreparedFile {
  file: FileDiff
  fidx: number
  hunks: PreparedHunk[]
}

/** Refine paired del→add lines with an intra-line word diff. A maximal run of
 * `del` immediately followed by a run of `add` is a replacement block; line
 * `p` of each run is paired for the highlight. Unpaired adds/dels stay plain. */
function applyWordDiff(lines: PreparedLine[]): void {
  let k = 0
  while (k < lines.length) {
    if (lines[k].line.kind === 'del') {
      const dels: number[] = []
      while (k < lines.length && lines[k].line.kind === 'del') {
        dels.push(k)
        k++
      }
      const adds: number[] = []
      while (k < lines.length && lines[k].line.kind === 'add') {
        adds.push(k)
        k++
      }
      const pairs = Math.min(dels.length, adds.length)
      for (let p = 0; p < pairs; p++) {
        const wd = wordDiff(lines[dels[p]].line.content, lines[adds[p]].line.content)
        lines[dels[p]].words = wd.old
        lines[adds[p]].words = wd.new
      }
    } else {
      k++
    }
  }
}

function prepare(files: FileDiff[]): { prepared: PreparedFile[]; hunkCount: number } {
  let gid = 0
  const prepared = files.map((file, fidx) => ({
    file,
    fidx,
    hunks: file.hunks.map((hunk) => {
      const lines: PreparedLine[] = hunk.lines.map((line) => ({ line }))
      applyWordDiff(lines)
      return { header: hunk.header, lines, gid: gid++ }
    }),
  }))
  return { prepared, hunkCount: gid }
}

interface SplitRow {
  left?: PreparedLine
  right?: PreparedLine
}

/** Lay a unified hunk out as side-by-side rows: ctx on both sides, a del/add
 * replacement block zipped row by row, a lone add on the right. */
function splitRows(lines: PreparedLine[]): SplitRow[] {
  const rows: SplitRow[] = []
  let k = 0
  while (k < lines.length) {
    const kind = lines[k].line.kind
    if (kind === 'ctx') {
      rows.push({ left: lines[k], right: lines[k] })
      k++
    } else if (kind === 'del') {
      const dels: PreparedLine[] = []
      while (k < lines.length && lines[k].line.kind === 'del') {
        dels.push(lines[k])
        k++
      }
      const adds: PreparedLine[] = []
      while (k < lines.length && lines[k].line.kind === 'add') {
        adds.push(lines[k])
        k++
      }
      const max = Math.max(dels.length, adds.length)
      for (let i = 0; i < max; i++) rows.push({ left: dels[i], right: adds[i] })
    } else {
      rows.push({ right: lines[k] })
      k++
    }
  }
  return rows
}

function lineTone(kind: DiffLine['kind']): string {
  if (kind === 'add') return 'bg-ok-bg/40 text-tx'
  if (kind === 'del') return 'bg-bad-bg/40 text-tx'
  return 'text-tx3'
}

function gutter(n: number | null): string {
  return n === null ? '' : String(n)
}

/** The line content — plain, or with the word-diff segments highlighted. */
function LineContent({ line }: { line: PreparedLine }) {
  if (!line.words) return <span className="whitespace-pre">{line.line.content}</span>
  const hl = line.line.kind === 'del' ? 'bg-bad-bg text-tx' : 'bg-ok-bg text-tx'
  return (
    <span className="whitespace-pre">
      {line.words.map((seg, i) =>
        seg.changed ? (
          <span key={i} data-testid="word-changed" className={`rounded-[1px] ${hl}`}>
            {seg.text}
          </span>
        ) : (
          <span key={i}>{seg.text}</span>
        ),
      )}
    </span>
  )
}

function marker(kind: DiffLine['kind']): string {
  return kind === 'add' ? '+' : kind === 'del' ? '−' : ' '
}

function InlineRow({ line }: { line: PreparedLine }) {
  return (
    <div className={`flex font-mono text-[10.5px] leading-relaxed ${lineTone(line.line.kind)}`}>
      <span className="w-10 flex-shrink-0 select-none px-1 text-right text-tx4 tabular-nums">
        {gutter(line.line.old_lineno)}
      </span>
      <span className="w-10 flex-shrink-0 select-none px-1 text-right text-tx4 tabular-nums">
        {gutter(line.line.new_lineno)}
      </span>
      <span className="w-3 flex-shrink-0 select-none text-center text-tx4" aria-hidden>
        {marker(line.line.kind)}
      </span>
      <LineContent line={line} />
    </div>
  )
}

function SplitCell({ line, side }: { line?: PreparedLine; side: 'old' | 'new' }) {
  if (!line) return <div className="flex-1 bg-s1/40" aria-hidden />
  const n = side === 'old' ? line.line.old_lineno : line.line.new_lineno
  return (
    <div className={`flex flex-1 font-mono text-[10.5px] leading-relaxed ${lineTone(line.line.kind)}`}>
      <span className="w-9 flex-shrink-0 select-none px-1 text-right text-tx4 tabular-nums">{gutter(n)}</span>
      <span className="w-3 flex-shrink-0 select-none text-center text-tx4" aria-hidden>
        {marker(line.line.kind)}
      </span>
      <LineContent line={line} />
    </div>
  )
}

export function DiffViewer({ files, caption, emptyLabel, truncated, onHunkIntent, testid }: DiffViewerProps) {
  const [mode, setMode] = useState<DiffMode>('inline')
  const [changeSetOpen, setChangeSetOpen] = useState(true)
  const [current, setCurrent] = useState(0)

  const { prepared, hunkCount } = useMemo(() => prepare(files), [files])

  const fileRefs = useRef<(HTMLDivElement | null)[]>([])
  const hunkRefs = useRef<(HTMLDivElement | null)[]>([])
  const scrollRef = useRef<HTMLDivElement>(null)
  const autofocused = useRef(false)

  // Auto-focus the scroll area ONCE when content first appears, so j/k · ↑/↓
  // hunk navigation works without a preliminary click — unless the operator is
  // already typing somewhere (never steal focus from a text field).
  useEffect(() => {
    if (files.length === 0 || autofocused.current) return
    const active = document.activeElement
    const typing =
      active instanceof HTMLElement &&
      (active.tagName === 'INPUT' || active.tagName === 'TEXTAREA' || active.isContentEditable)
    if (!typing) {
      scrollRef.current?.focus({ preventScroll: true })
      autofocused.current = true
    }
  }, [files.length])

  const totalInsertions = files.reduce((a, f) => a + f.insertions, 0)
  const totalDeletions = files.reduce((a, f) => a + f.deletions, 0)

  function scrollToFile(fidx: number) {
    fileRefs.current[fidx]?.scrollIntoView?.({ block: 'start' })
  }

  function moveHunk(delta: number) {
    if (hunkCount === 0) return
    setCurrent((c) => {
      const next = Math.min(hunkCount - 1, Math.max(0, c + delta))
      hunkRefs.current[next]?.scrollIntoView?.({ block: 'center' })
      return next
    })
  }

  function onKeyDown(e: React.KeyboardEvent<HTMLDivElement>) {
    if (e.key === 'ArrowDown' || e.key === 'j') {
      e.preventDefault()
      moveHunk(1)
    } else if (e.key === 'ArrowUp' || e.key === 'k') {
      e.preventDefault()
      moveHunk(-1)
    }
  }

  if (files.length === 0) {
    return (
      <div data-testid={testid ?? 'verify-diff'} className="flex flex-1 flex-col">
        {caption ? <div className="border-b border-bd px-4 py-2 font-mono text-[10.5px] text-tx3">{caption}</div> : null}
        <div className="flex flex-1 items-center justify-center p-8 font-mono text-[11px] text-tx4">
          {emptyLabel ?? 'aucun changement dans l’arbre de travail'}
        </div>
      </div>
    )
  }

  return (
    <div data-testid={testid ?? 'verify-diff'} className="flex min-h-0 flex-1 flex-col">
      {/* toolbar: source caption + bi-mode toggle + change-set toggle + counts */}
      <div className="flex items-center gap-3 border-b border-bd bg-s1 px-4 py-2 font-mono text-[10.5px] text-tx3">
        {caption ? <span className="truncate text-tx2">{caption}</span> : null}
        <span className="text-ok">+{totalInsertions}</span>
        <span className="text-bad">−{totalDeletions}</span>
        {truncated ? (
          <span className="text-warn" title="diff coupé à 20 000 lignes par le backend">
            ◦ tronqué
          </span>
        ) : null}
        <div className="ml-auto flex items-center gap-1.5">
          <button
            type="button"
            data-testid="changeset-toggle"
            aria-pressed={changeSetOpen}
            onClick={() => setChangeSetOpen((v) => !v)}
            className="rounded-sm px-2 py-0.5 text-tx3 hover:bg-s2"
            title={changeSetOpen ? 'replier → diff plein' : 'afficher le change-set'}
          >
            change-set {changeSetOpen ? '◂' : '▸'}
          </button>
          <div className="flex overflow-hidden rounded-sm border border-bd" role="group" aria-label="Disposition du diff">
            <button
              type="button"
              data-testid="diff-mode-inline"
              aria-pressed={mode === 'inline'}
              onClick={() => setMode('inline')}
              className={mode === 'inline' ? 'bg-s3 px-2 py-0.5 text-tx' : 'px-2 py-0.5 text-tx3 hover:bg-s2'}
            >
              Inline
            </button>
            <button
              type="button"
              data-testid="diff-mode-split"
              aria-pressed={mode === 'split'}
              onClick={() => setMode('split')}
              className={mode === 'split' ? 'bg-s3 px-2 py-0.5 text-tx' : 'px-2 py-0.5 text-tx3 hover:bg-s2'}
            >
              Côte à côte
            </button>
          </div>
        </div>
      </div>

      <div className="flex min-h-0 flex-1">
        {/* change-set column (collapsible → diff plein) */}
        {changeSetOpen ? (
          <div
            data-testid="diff-changeset"
            className="flex w-52 flex-shrink-0 flex-col overflow-auto border-r border-bd bg-s1"
          >
            <div className="border-b border-bd px-3 py-1.5 font-sans text-[8.5px] font-semibold uppercase tracking-wider text-tx4">
              change-set · {files.length}
            </div>
            {prepared.map((pf) => (
              <button
                key={pf.fidx}
                type="button"
                data-testid="changeset-file"
                onClick={() => scrollToFile(pf.fidx)}
                title={pf.file.path}
                className="flex items-center gap-2 border-b border-bd/50 px-3 py-1.5 text-left font-mono text-[10px] hover:bg-s2"
              >
                {/* per-file gate marker is degraded to S81 (the gate `file` is a
                   .planning basename, not a change-set path); a neutral dot
                   marks "in the change-set", never a fabricated gate verdict. */}
                <span className="h-1.5 w-1.5 flex-shrink-0 rounded-full bg-tx4" aria-hidden />
                <span className="min-w-0 flex-1 truncate text-tx2">{pf.file.path}</span>
                <span className="text-ok">+{pf.file.insertions}</span>
                <span className="text-bad">−{pf.file.deletions}</span>
              </button>
            ))}
            <div className="mt-auto px-3 py-2 font-mono text-[8px] leading-relaxed text-tx4">
              marqueur de gate par fichier · dégradé S81
            </div>
          </div>
        ) : null}

        {/* diff scroll area — keyboard navigable (j/k · ↑/↓ between hunks) */}
        <div
          ref={scrollRef}
          data-testid="diff-scroll"
          tabIndex={0}
          role="group"
          aria-label="Diff — naviguer entre les hunks avec les flèches ou j/k"
          onKeyDown={onKeyDown}
          className="min-h-0 flex-1 overflow-auto outline-none"
        >
          {prepared.map((pf) => (
            <div
              key={pf.fidx}
              data-testid="diff-file"
              ref={(el) => {
                fileRefs.current[pf.fidx] = el
              }}
              className="border-b border-bd"
            >
              <div className="sticky top-0 z-[1] flex items-center gap-2 border-b border-bd bg-s2 px-3 py-1.5 font-mono text-[10.5px]">
                <span className="min-w-0 flex-1 truncate text-tx">{pf.file.path}</span>
                <span className="text-ok">+{pf.file.insertions}</span>
                <span className="text-bad">−{pf.file.deletions}</span>
              </div>
              {pf.hunks.map((hunk) => (
                <div
                  key={hunk.gid}
                  data-testid="diff-hunk"
                  aria-current={hunk.gid === current}
                  ref={(el) => {
                    hunkRefs.current[hunk.gid] = el
                  }}
                  className={hunk.gid === current ? 'bg-info/5 ring-1 ring-inset ring-info/30' : ''}
                >
                  <div className="flex items-center gap-2 bg-s1 px-3 py-1 font-mono text-[10px] text-info">
                    <span className="min-w-0 flex-1 truncate">{hunk.header}</span>
                    {onHunkIntent ? (
                      <button
                        type="button"
                        data-testid="hunk-intent"
                        onClick={() => onHunkIntent(pf.file.path, hunk.header)}
                        className="flex-shrink-0 rounded-sm px-1.5 py-0.5 text-[9.5px] text-tx3 hover:bg-s3 hover:text-tx2"
                        title="transmettre cette correction à la session (ne l’exécute pas)"
                      >
                        → transmettre à la session
                      </button>
                    ) : null}
                  </div>
                  {mode === 'inline'
                    ? hunk.lines.map((line, li) => <InlineRow key={li} line={line} />)
                    : splitRows(hunk.lines).map((row, ri) => (
                        <div key={ri} className="flex">
                          <SplitCell line={row.left} side="old" />
                          <span className="w-px flex-shrink-0 bg-bd" aria-hidden />
                          <SplitCell line={row.right} side="new" />
                        </div>
                      ))}
                </div>
              ))}
            </div>
          ))}
        </div>

        {/* density minimap (fold V3) — click a file bar to jump to it */}
        <div
          data-testid="diff-minimap"
          className="flex w-7 flex-shrink-0 flex-col gap-1 overflow-hidden border-l border-bd bg-s1 px-1 py-2"
          aria-label="Densité des changements par fichier"
        >
          {prepared.map((pf) => (
            <button
              key={pf.fidx}
              type="button"
              onClick={() => scrollToFile(pf.fidx)}
              title={`${pf.file.path} · +${pf.file.insertions} −${pf.file.deletions}`}
              className="flex h-6 w-full flex-col overflow-hidden rounded-[2px] border border-bd/60 hover:border-info"
            >
              <span className="w-full bg-ok" style={{ flexGrow: pf.file.insertions + 1 }} aria-hidden />
              <span className="w-full bg-bad" style={{ flexGrow: pf.file.deletions + 1 }} aria-hidden />
            </button>
          ))}
        </div>
      </div>
    </div>
  )
}
