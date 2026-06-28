// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase D — the commit conformity card (folds U3/A9/V10). It
// restitutes the Rust audit of a commit (/api/audit/{rev}: missing review /
// PASS / Codex artifacts) and the planning lint (/api/lint). The cardinal
// rule: issues are listed as "manques" (what is missing) — NEVER rendered as a
// green tick of approval. "0 manque relevé" is the honest empty state, not a
// verdict the Operator grants.
import { useEffect, useState } from 'react'
import { getAudit, getLint, OperatorError, type AuditCommit, type Lint } from '../../api/operator'

interface Resolved {
  rev: string
  audit: AuditCommit | null
  lint: Lint | null
  error: string | null
}

export function ConformiteCard({ rev }: { rev: string }) {
  // State written ONLY from the async resolution; the loading/reset state is
  // derived by comparing the resolved rev to the requested one (react-hooks).
  const [resolved, setResolved] = useState<Resolved | null>(null)

  useEffect(() => {
    const controller = new AbortController()
    Promise.allSettled([getAudit(rev, controller.signal), getLint(controller.signal)]).then(
      ([auditRes, lintRes]) => {
        if (controller.signal.aborted) return
        setResolved({
          rev,
          audit: auditRes.status === 'fulfilled' ? auditRes.value : null,
          lint: lintRes.status === 'fulfilled' ? lintRes.value : null,
          error:
            auditRes.status === 'rejected'
              ? auditRes.reason instanceof OperatorError
                ? `audit indisponible (${auditRes.reason.status})`
                : 'audit indisponible'
              : null,
        })
      },
    )
    return () => controller.abort()
  }, [rev])

  const ready = resolved !== null && resolved.rev === rev
  const audit = ready ? resolved.audit : null
  const lint = ready ? resolved.lint : null
  const error = ready ? resolved.error : null

  if (error) return <div className="font-mono text-[10.5px] text-warn">{error}</div>
  if (audit === null) return <div className="font-mono text-[10.5px] text-tx4">audit de conformité…</div>

  return (
    <div data-testid="conformite-card" className="flex flex-col gap-3 rounded-md border border-bd bg-s1 px-4 py-3">
      <div className="flex items-center gap-2 font-mono text-[10.5px]">
        <span className="eyebrow">
          conformité du commit
        </span>
        <span className="text-tx">{audit.rev.slice(0, 10)}</span>
        {audit.is_phase_commit ? (
          <span className="rounded-sm border border-bd2 px-1 py-0.5 text-[12px] text-tx3">phase</span>
        ) : (
          <span className="rounded-sm border border-bd px-1 py-0.5 text-[12px] text-tx3">hors-phase</span>
        )}
      </div>

      <div>
        <div className="mb-1 font-mono text-[9px] uppercase tracking-wider text-tx4">
          manques relevés <span className="text-tx3">· {audit.issues.length}</span>
        </div>
        {audit.issues.length === 0 ? (
          <div className="font-mono text-[10.5px] text-tx3">0 manque relevé par l'audit Rust</div>
        ) : (
          <ul className="flex flex-col gap-1">
            {audit.issues.map((issue, i) => (
              <li key={i} className="flex items-start gap-2 font-mono text-[10.5px] text-warn">
                <span aria-hidden>−</span>
                <span className="text-tx2">{issue}</span>
              </li>
            ))}
          </ul>
        )}
      </div>

      {lint ? (
        <div className="border-t border-bd pt-2">
          <div className="mb-1 font-mono text-[9px] uppercase tracking-wider text-tx4">
            lint planning <span className="text-tx3">· {lint.errors.length} err · {lint.warnings.length} warn</span>
          </div>
          {lint.errors.length === 0 && lint.warnings.length === 0 ? (
            <div className="font-mono text-[10.5px] text-tx3">0 diagnostic</div>
          ) : (
            <ul className="flex flex-col gap-1">
              {lint.errors.map((d, i) => (
                <li key={`e${i}`} className="font-mono text-[10px] text-bad">
                  [{d.code}] {d.message}
                  {d.file ? <span className="text-tx4"> · {d.file}</span> : null}
                </li>
              ))}
              {lint.warnings.map((d, i) => (
                <li key={`w${i}`} className="font-mono text-[10px] text-warn">
                  [{d.code}] {d.message}
                  {d.file ? <span className="text-tx4"> · {d.file}</span> : null}
                </li>
              ))}
            </ul>
          )}
        </div>
      ) : null}
    </div>
  )
}
