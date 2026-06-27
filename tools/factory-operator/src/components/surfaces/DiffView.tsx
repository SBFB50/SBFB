// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase D — a read-only viewer for a PAST commit's diff (J11). The
// hunks are computed IN RUST (sprint_history.rs parse_unified_diff) — the
// single source of truth, never a JS re-diff (kickoff invariant #11). Phase H
// builds the bespoke bi-mode (inline ⇄ side-by-side) viewer on the SAME shape
// (it also serves the working-tree `/api/git/diff`); this is the inline form.
import type { CommitDiff, DiffLine } from '../../api/operator'

function lineTone(kind: DiffLine['kind']): string {
  if (kind === 'add') return 'bg-ok-bg/40 text-tx'
  if (kind === 'del') return 'bg-bad-bg/40 text-tx'
  return 'text-tx3'
}

function gutter(n: number | null): string {
  return n === null ? '' : String(n)
}

export function DiffView({ diff }: { diff: CommitDiff }) {
  return (
    <div data-testid="diff-view" className="flex flex-col gap-3">
      <div className="font-mono text-[11px] text-tx2">
        <span className="text-tx4">commit</span> <span className="text-tx">{diff.sha.slice(0, 10)}</span>{' '}
        <span className="text-tx3">— {diff.title}</span>
      </div>
      {diff.files.length === 0 ? (
        <div className="font-mono text-[10.5px] text-tx4">aucun fichier dans ce diff</div>
      ) : (
        diff.files.map((file) => (
          <div key={file.path} className="overflow-hidden rounded-md border border-bd">
            <div className="flex items-center gap-2 border-b border-bd bg-s2 px-3 py-1.5 font-mono text-[10.5px]">
              <span className="min-w-0 flex-1 truncate text-tx">{file.path}</span>
              <span className="text-ok">+{file.insertions}</span>
              <span className="text-bad">−{file.deletions}</span>
            </div>
            <div className="overflow-x-auto">
              {file.hunks.map((hunk, hi) => (
                <div key={hi}>
                  <div className="bg-s1 px-3 py-1 font-mono text-[10px] text-info">{hunk.header}</div>
                  {hunk.lines.map((line, li) => (
                    <div key={li} className={`flex font-mono text-[10.5px] leading-relaxed ${lineTone(line.kind)}`}>
                      <span className="w-10 flex-shrink-0 select-none px-1 text-right text-tx4 tabular-nums">
                        {gutter(line.old_lineno)}
                      </span>
                      <span className="w-10 flex-shrink-0 select-none px-1 text-right text-tx4 tabular-nums">
                        {gutter(line.new_lineno)}
                      </span>
                      <span className="w-3 flex-shrink-0 select-none text-center text-tx4" aria-hidden>
                        {line.kind === 'add' ? '+' : line.kind === 'del' ? '−' : ' '}
                      </span>
                      <span className="whitespace-pre">{line.content}</span>
                    </div>
                  ))}
                </div>
              ))}
            </div>
          </div>
        ))
      )}
    </div>
  )
}
