// SPDX-License-Identifier: AGPL-3.0-or-later
import { useDeferredValue, useEffect, useMemo, useState } from 'react'
import { Eye, FileText, Link2, PenLine, Pin, RefreshCw, ScanLine, Search } from 'lucide-react'
import {
  getProjectDocuments,
  OperatorError,
  type ProjectDocumentCard,
  type ProjectDocumentPinned,
  type ProjectDocumentRole,
  type ProjectDocuments,
} from '../../api/operator'
import { cn } from '../../lib/cn'
import { AdaptiveSurface } from '../AdaptiveSurface'

const LIVE_INTERVAL_MS = 1_800
const EMPTY_DOCUMENTS: ProjectDocumentCard[] = []

type SortMode = 'activity' | 'path' | 'kind' | 'modified'
type RoleFilter = ProjectDocumentRole | 'all'

const ROLE_ORDER: ProjectDocumentRole[] = ['write', 'read', 'use', 'scan']
const ROLE_META: Record<ProjectDocumentRole, {
  label: string
  borderColor: string
  text: string
  Icon: typeof PenLine
}> = {
  write: { label: 'écrit', borderColor: 'var(--color-bad)', text: 'text-bad', Icon: PenLine },
  read: { label: 'lit', borderColor: 'var(--color-info)', text: 'text-info', Icon: Eye },
  use: { label: 'utilise', borderColor: 'var(--color-ok)', text: 'text-ok', Icon: Link2 },
  scan: { label: 'scan', borderColor: 'var(--color-bd2)', text: 'text-tx3', Icon: ScanLine },
}

function primaryRole(roles: ProjectDocumentRole[]): ProjectDocumentRole {
  return ROLE_ORDER.find((role) => roles.includes(role)) ?? 'scan'
}

function roleRank(roles: ProjectDocumentRole[]): number {
  return ROLE_ORDER.indexOf(primaryRole(roles))
}

function formatBytes(value: number | null): string {
  if (value === null) return '—'
  if (value < 1024) return `${value} o`
  if (value < 1024 * 1024) return `${Math.round(value / 1024)} ko`
  return `${(value / 1024 / 1024).toFixed(1)} Mo`
}

function formatTime(value: number | null): string {
  if (value === null) return ''
  return new Intl.DateTimeFormat('fr-FR', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  }).format(value)
}

function RolePill({ role }: { role: ProjectDocumentRole }) {
  const meta = ROLE_META[role]
  const Icon = meta.Icon
  return (
    <span className={cn('inline-flex items-center gap-1 rounded-sm border border-bd px-1.5 py-0.5 font-mono text-meta', meta.text)}>
      <Icon className="h-3 w-3" aria-hidden />
      {meta.label}
    </span>
  )
}

function PinnedItem({ item }: { item: ProjectDocumentPinned }) {
  const meta = ROLE_META[item.role]
  const Icon = meta.Icon
  return (
    <li className="bg-s1 px-2.5 py-2" style={{ borderLeft: `4px solid ${meta.borderColor}` }}>
      <div className="flex items-center gap-2">
        <Icon className={cn('h-3.5 w-3.5 shrink-0', meta.text)} aria-hidden />
        <span className="min-w-0 flex-1 truncate font-mono text-meta text-tx2">{item.path}</span>
        <span className="shrink-0 font-mono text-meta text-tx4">{item.label}</span>
      </div>
      <div className="mt-0.5 flex gap-2 font-mono text-meta text-tx4">
        <span>{item.source}</span>
        <span className="min-w-0 truncate">{item.detail}</span>
      </div>
    </li>
  )
}

function DocumentCardView({ doc }: { doc: ProjectDocumentCard }) {
  const role = primaryRole(doc.roles)
  const meta = ROLE_META[role]
  return (
    <article
      data-testid="project-document-card"
      className="border border-bd bg-s1 px-3 py-2.5"
      style={{
        borderLeft: `4px solid ${meta.borderColor}`,
        contentVisibility: 'auto',
        containIntrinsicSize: '120px',
      }}
    >
      <div className="flex items-start gap-2">
        <FileText className="mt-0.5 h-4 w-4 shrink-0 text-tx4" aria-hidden />
        <div className="min-w-0 flex-1">
          <div className="truncate font-mono text-sec text-tx">{doc.name}</div>
          <div className="truncate font-mono text-meta text-tx4">{doc.dir || 'racine'}</div>
        </div>
        <div className="flex shrink-0 flex-wrap justify-end gap-1">
          {doc.roles.map((r) => <RolePill key={r} role={r} />)}
        </div>
      </div>

      <div className="mt-2 grid grid-cols-2 gap-1 font-mono text-meta text-tx4">
        <span>{doc.kind}</span>
        <span className="text-right">{formatBytes(doc.size_bytes)}</span>
        <span>{doc.tracked ? 'suivi git' : 'non suivi'}</span>
        <span className="text-right">{doc.status || formatTime(doc.modified_ms)}</span>
      </div>

      <div className="mt-2 flex flex-col gap-0.5">
        {doc.sources.slice(0, 3).map((source, i) => (
          <div key={`${source.role}-${source.source}-${i}`} className="flex gap-1.5 font-mono text-meta text-tx4">
            <span className={ROLE_META[source.role].text}>{ROLE_META[source.role].label}</span>
            <span className="truncate">{source.source}</span>
            <span className="min-w-0 flex-1 truncate">{source.detail}</span>
          </div>
        ))}
        {doc.sources.length > 3 ? (
          <div className="font-mono text-meta text-tx4">+ {doc.sources.length - 3} sources</div>
        ) : null}
      </div>
    </article>
  )
}

function filterDocuments(
  documents: ProjectDocumentCard[],
  query: string,
  role: RoleFilter,
  kind: string,
  sort: SortMode,
): ProjectDocumentCard[] {
  const q = query.trim().toLowerCase()
  const filtered = documents.filter((doc) => {
    if (role !== 'all' && !doc.roles.includes(role)) return false
    if (kind !== 'all' && doc.kind !== kind) return false
    if (!q) return true
    return doc.path.toLowerCase().includes(q) || doc.sources.some((s) => s.source.toLowerCase().includes(q))
  })
  return [...filtered].sort((a, b) => {
    if (sort === 'activity') {
      return roleRank(a.roles) - roleRank(b.roles) || a.path.localeCompare(b.path)
    }
    if (sort === 'kind') return a.kind.localeCompare(b.kind) || a.path.localeCompare(b.path)
    if (sort === 'modified') return (b.modified_ms ?? 0) - (a.modified_ms ?? 0) || a.path.localeCompare(b.path)
    return a.path.localeCompare(b.path)
  })
}

export function DocumentsSurface({ sessionId }: { sessionId: string | null }) {
  const [snapshot, setSnapshot] = useState<ProjectDocuments | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [query, setQuery] = useState('')
  const [role, setRole] = useState<RoleFilter>('all')
  const [kind, setKind] = useState('all')
  const [sort, setSort] = useState<SortMode>('activity')
  const deferredQuery = useDeferredValue(query)

  useEffect(() => {
    let stopped = false
    let inFlight = false
    let controller: AbortController | null = null
    const load = () => {
      if (inFlight) return
      controller = new AbortController()
      inFlight = true
      getProjectDocuments(sessionId, controller.signal)
        .then((data) => {
          if (stopped) return
          setSnapshot(data)
          setError(null)
        })
        .catch((err) => {
          if (stopped || controller?.signal.aborted) return
          setError(err instanceof OperatorError ? `documents indisponibles (${err.status})` : 'documents indisponibles')
        })
        .finally(() => {
          inFlight = false
        })
    }
    load()
    const timer = window.setInterval(load, LIVE_INTERVAL_MS)
    return () => {
      stopped = true
      if (controller !== null) controller.abort()
      window.clearInterval(timer)
    }
  }, [sessionId])

  const documents = snapshot?.documents ?? EMPTY_DOCUMENTS
  const roleCounts = useMemo(
    () => Object.fromEntries(ROLE_ORDER.map((r) => [r, documents.filter((doc) => doc.roles.includes(r)).length])),
    [documents],
  ) as Record<ProjectDocumentRole, number>
  const kinds = useMemo(() => [...new Set(documents.map((doc) => doc.kind))].sort(), [documents])
  const visibleDocuments = useMemo(
    () => filterDocuments(documents, deferredQuery, role, kind, sort),
    [documents, deferredQuery, role, kind, sort],
  )
  const pinned = snapshot?.pinned ?? []

  return (
    <AdaptiveSurface kind="documents" testId="documents-surface" className="flex min-h-0 flex-1 flex-col overflow-hidden p-5">
      <div className="adaptive-surface-header mb-4 flex items-start gap-3">
        <div className="min-w-0 flex-1">
          <h2 className="font-sans text-card font-semibold text-tx">Documents projet</h2>
          <div className="font-mono text-meta text-tx4">
            {snapshot ? `${snapshot.total} fichiers · ${snapshot.branch}@${snapshot.head}` : 'lecture du dépôt…'}
          </div>
        </div>
        <div className="flex shrink-0 items-center gap-2 font-mono text-meta text-tx4">
          <span data-testid="documents-live-stamp">{snapshot ? formatTime(Date.parse(snapshot.generated_at)) : '—'}</span>
          <RefreshCw className="h-3.5 w-3.5" aria-hidden />
        </div>
      </div>

      <div
        className="mb-4 grid gap-3"
        style={{ gridTemplateColumns: 'repeat(auto-fit, minmax(min(100%, 22rem), 1fr))' }}
      >
        <section className="min-h-0 rounded-md border border-bd bg-s0 p-3">
          <div className="mb-2 flex items-center gap-2 eyebrow">
            <Pin className="h-3.5 w-3.5" aria-hidden />
            suivi épinglé
          </div>
          {snapshot?.session ? (
            <div className="mb-2 rounded-sm border border-bd bg-s1 px-2.5 py-2 font-mono text-meta text-tx4">
              <span className="text-tx2">{snapshot.session.provider}</span> · {snapshot.session.model} ·{' '}
              {snapshot.session.messages} messages · chat non-autoritaire
            </div>
          ) : null}
          <ul className="flex flex-col gap-1 overflow-auto" style={{ maxHeight: '20rem' }} data-testid="documents-pinned">
            {pinned.length === 0 ? (
              <li className="font-mono text-meta text-tx4">aucun fichier épinglé</li>
            ) : (
              pinned.map((item) => <PinnedItem key={`${item.role}-${item.source}-${item.path}`} item={item} />)
            )}
          </ul>
        </section>

        <section className="min-w-0 rounded-md border border-bd bg-s0 p-3">
          <div className="adaptive-surface-toolbar mb-3 flex items-center gap-2">
            <label className="flex min-w-0 flex-1 items-center gap-2 rounded-sm border border-field bg-s1 px-2.5 py-1.5">
              <Search className="h-4 w-4 shrink-0 text-tx4" aria-hidden />
              <input
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                aria-label="recherche fichiers"
                placeholder="chemin, source, rôle"
                className="min-w-0 flex-1 bg-transparent font-sans text-body text-tx outline-none placeholder:text-tx4"
              />
            </label>
            <select
              value={kind}
              onChange={(event) => setKind(event.target.value)}
              aria-label="type"
              className="rounded-sm border border-field bg-s1 px-2 py-1.5 font-mono text-meta text-tx2"
            >
              <option value="all">types</option>
              {kinds.map((k) => <option key={k} value={k}>{k}</option>)}
            </select>
            <select
              value={sort}
              onChange={(event) => setSort(event.target.value as SortMode)}
              aria-label="tri"
              className="rounded-sm border border-field bg-s1 px-2 py-1.5 font-mono text-meta text-tx2"
            >
              <option value="activity">activité</option>
              <option value="path">chemin</option>
              <option value="kind">type</option>
              <option value="modified">modifié</option>
            </select>
          </div>

          <div className="adaptive-surface-toolbar mb-3 flex flex-wrap gap-1.5">
            <button
              type="button"
              onClick={() => setRole('all')}
              className={cn('rounded-sm border px-2 py-1 font-mono text-meta', role === 'all' ? 'border-bd2 bg-s2 text-tx' : 'border-bd text-tx3 hover:border-bd2')}
            >
              tous <span className="text-tx4">· {documents.length}</span>
            </button>
            {ROLE_ORDER.map((r) => (
              <button
                key={r}
                type="button"
                onClick={() => setRole(r)}
                className={cn('rounded-sm border px-2 py-1 font-mono text-meta', role === r ? 'border-bd2 bg-s2 text-tx' : 'border-bd text-tx3 hover:border-bd2')}
              >
                {ROLE_META[r].label} <span className="text-tx4">· {roleCounts[r]}</span>
              </button>
            ))}
          </div>

          {error ? (
            <div className="rounded-sm border border-bd bg-s1 px-3 py-2 font-mono text-meta text-warn">{error}</div>
          ) : snapshot === null ? (
            <div className="rounded-sm border border-bd bg-s1 px-3 py-2 font-mono text-meta text-tx4">lecture live…</div>
          ) : (
            <div className="font-mono text-meta text-tx4">
              {visibleDocuments.length} affichés{snapshot.truncated ? ' · réponse tronquée côté serveur' : ''}
            </div>
          )}
        </section>
      </div>

      <div className="min-h-0 flex-1 overflow-auto pr-1">
        <div
          className="grid gap-2"
          style={{ gridTemplateColumns: 'repeat(auto-fit, minmax(min(100%, 21rem), 1fr))' }}
        >
          {visibleDocuments.map((doc) => <DocumentCardView key={doc.path} doc={doc} />)}
        </div>
      </div>
    </AdaptiveSurface>
  )
}
