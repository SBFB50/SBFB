// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C — typed client for the Operator backend (crate
// `sbfb-factory`, 0 daemon route). Every call is same-origin and relative:
// in prod the SPA is served by the Operator's ServeDir and authenticates
// via the HttpOnly `sbfb_operator` cookie (Phase A `a5ace8d`); in dev the
// Vite proxy injects the bearer header server-to-server. The browser
// attaches the cookie automatically — we set `credentials: 'same-origin'`
// (the default, made explicit) and NEVER `'omit'` (that would strip the
// cookie → 401) and NEVER read a token in JS (HttpOnly, unreadable).
// Preflight S3 / plan-adaptation #6.

/** A non-2xx Operator response. `status` is the HTTP code (e.g. 401, 404). */
export class OperatorError extends Error {
  readonly status: number
  readonly path: string
  constructor(status: number, path: string) {
    super(`operator ${path} → ${status}`)
    this.name = 'OperatorError'
    this.status = status
    this.path = path
  }
}

async function getJson<T>(path: string, signal?: AbortSignal): Promise<T> {
  const res = await fetch(path, {
    credentials: 'same-origin',
    headers: { accept: 'application/json' },
    signal,
  })
  if (!res.ok) throw new OperatorError(res.status, path)
  return (await res.json()) as T
}

async function postJson<T>(path: string, body: unknown): Promise<T> {
  const res = await fetch(path, {
    method: 'POST',
    credentials: 'same-origin',
    headers: { 'content-type': 'application/json', accept: 'application/json' },
    body: JSON.stringify(body),
  })
  if (!res.ok) throw new OperatorError(res.status, path)
  return (await res.json()) as T
}

/** Plain-text GET (a `.cast` recording is served as raw text, not JSON). */
async function getText(path: string, signal?: AbortSignal): Promise<string> {
  const res = await fetch(path, { credentials: 'same-origin', signal })
  if (!res.ok) throw new OperatorError(res.status, path)
  return res.text()
}

// --- response shapes (mirror operator_server.rs / process.rs) ---

/** `GET /api/context` — carries the working-tree counts the rail needs. */
export interface RepoContext {
  branch: string
  head: string
  sprint?: number
  phase?: string
  dirty_files: string[]
  staged_files: string[]
}

/** `POST /api/chat/session` → `{ id, context_pack }`. */
export interface SessionCreated {
  id: string
  context_pack: unknown
}

/**
 * `POST /api/chat/{id}/send` → `{ ok }` or `{ ok: false, requires_gate: true }`.
 * The send appends the user message and runs the SENSITIVE_ACTIONS keyword
 * gate BEFORE any spawn; a sensitive message returns `requires_gate` and the
 * front restitutes the MUR without ever opening the stream (0 spawn).
 */
export interface SendResult {
  ok: boolean
  requires_gate?: boolean
  provider?: string
}

/** `GET /api/prompt/{kind}` — the real assembled prompt (inspector). */
export interface PromptInspect {
  kind: string
  provider: string
  content: string
}

// --- calls ---

export function getContext(signal?: AbortSignal): Promise<RepoContext> {
  return getJson<RepoContext>('/api/context', signal)
}

/**
 * S4 diagnostic probe: the backend's prompt-adaptation provider set. Used as a
 * reachability signal for the rail, NOT as the chat execution axis — the chat
 * routes on `ExecutionTarget::from_provider` ({claude, local, network}), a
 * distinct axis from this list ({claude, codex, gpt, local, human}).
 */
export function getProviders(signal?: AbortSignal): Promise<{ providers: string[] }> {
  return getJson<{ providers: string[] }>('/api/providers', signal)
}

export function getPrompt(kind: string, provider: string, signal?: AbortSignal): Promise<PromptInspect> {
  const qs = new URLSearchParams({ provider }).toString()
  return getJson<PromptInspect>(`/api/prompt/${encodeURIComponent(kind)}?${qs}`, signal)
}

export function createSession(req: {
  provider: string
  intent: string
}): Promise<SessionCreated> {
  return postJson<SessionCreated>('/api/chat/session', req)
}

export function sendMessage(
  id: string,
  req: { message: string; provider: string; model?: string },
): Promise<SendResult> {
  return postJson<SendResult>(`/api/chat/${encodeURIComponent(id)}/send`, req)
}

/** The bodyless SSE GET. Auth rides the same-origin cookie automatically. */
export function streamUrl(id: string): string {
  return `/api/chat/${encodeURIComponent(id)}/stream`
}

// --- Sprint 80 Phase D — VERIFY-bootstrap + procédé surfaces ---
//
// Every shape below MIRRORS a Rust struct (sprint_history.rs / process.rs /
// operator_server.rs) and is RESTITUTED by the front, never computed: the
// arbre de procédé, the conformity card, the gates pulse — all carry a
// verdict GRAVED by Rust (kickoff cardinal: 0 verdict calculé UI).

/** A hashed path reference (`file_hash`): the sealed context-pack never
 * inlines content, only `{path, hash, exists}` — provenance, not payload. */
export interface HashRef {
  path: string
  hash?: string
  exists: boolean
}

/** One phase in the arbre de procédé (`PhaseHistory`). The verdicts are the
 * RESTITUTED record (preflight EXECUTE/PLAN-ADAPT…, review PASS…), read from
 * the on-disk artifacts — the UI asserts nothing. */
export interface PhaseHistory {
  letter: string
  title: string
  commit_sha: string | null
  commit_date: string | null
  commit_type: string | null
  preflight_verdict: string | null
  review_verdict: string | null
  codex_confirmed: number | null
  codex_partial: number | null
  codex_gap: number | null
  rust_delta: number
  vitest_delta: number
  files_changed: { path: string; insertions: number; deletions: number; status: string }[]
  deliverables: string[]
  findings: { severity: string; code: string; description: string; status: string }[]
}

/** `PreflightPhase` — the verdict AND its source artifact filename (U2
 * provenance-de-verdict: every verdict is clickable to its `.planning/` file). */
export interface PreflightPhase {
  phase: string
  verdict: string
  file: string
}

/** One commit in the sprint (`CommitInfo`). All fields are RESTITUTED from
 * `git log` parsing — `commit_type`/`is_phase`/`phase` are graved by the
 * backend regex, the UI never re-derives them. */
export interface CommitInfo {
  sha: string
  short: string
  title: string
  author: string
  date: string
  commit_type: string
  scope: string
  is_phase: boolean
  phase: string | null
  insertions: number
  deletions: number
  files: string[]
  body_sections: string[]
}

/** One §1 verification check (`VerificationCheck`). `result` is the RESTITUTED
 * historical PASS/FAIL recorded in verification.md — never a live verdict. */
export interface VerificationCheck {
  number: number
  name: string
  command: string
  result: string
}

/** `VerificationSummary` — the §1 fail-fast table read from verification.md.
 * `null` until the sprint is wrapped (no verification.md yet). */
export interface VerificationSummary {
  total_checks: number
  passed: number
  failed: number
  checks: VerificationCheck[]
}

/** A carried debt item (`CarryItem`) — open OR closed; `phase_closed` is set
 * only on a closed carry. Restituted disposition, never a UI judgement. */
export interface CarryItem {
  code: string
  description: string
  disposition: string
  phase_closed: string | null
}

export interface SprintHistory {
  sprint: number
  status: string
  branch: string
  head: string
  entry_tip: string | null
  exit_tip: string | null
  roadmap: string | null
  total_commits: number
  phase_commits: number
  chore_commits: number
  phases: PhaseHistory[]
  commits: CommitInfo[]
  preflight_bilan: {
    total: number
    execute: number
    plan_adapt: number
    design_conflict: number
    phases: PreflightPhase[]
  }
  tests: {
    rust_entry: number
    rust_exit: number
    rust_delta: number
    vitest_entry: number
    vitest_exit: number
    vitest_delta: number
    size_limit: string
    per_phase: { phase: string; rust_delta: number; vitest_delta: number; detail: string }[]
  }
  scope_cuts: { number: number; item: string; target: string; respected: boolean }[]
  carries_open: CarryItem[]
  carries_closed: CarryItem[]
  verification: VerificationSummary | null
}

/** One phase's live progress (`/api/status` phases[]) — artifact presence +
 * restituted review verdict, available BEFORE the phase commit lands. */
export interface StatusPhase {
  letter: string
  has_preflight: boolean
  has_review: boolean
  review_verdict: string | null
  has_codex: boolean
}

/** `GET /api/status` — the LIVE process position: which sprint/phase we are on
 * right now (`current_phase`), the planning-artifact presence, and per-phase
 * progress. Distinct from `/api/sprint-history` (committed history): this
 * surfaces the IN-PROGRESS phase before its commit exists. */
export interface OperatorStatus {
  sprint: number
  branch: string
  head: string
  current_phase: string | null
  has_kickoff: boolean
  has_plan: boolean
  has_design_review: boolean
  has_audit_plan: boolean
  phases: StatusPhase[]
}

export interface DiffLine {
  kind: 'add' | 'del' | 'ctx'
  content: string
  old_lineno: number | null
  new_lineno: number | null
}
export interface FileDiff {
  path: string
  insertions: number
  deletions: number
  hunks: { header: string; lines: DiffLine[] }[]
}
/** `CommitDiffResult` — the diff of a PAST commit (J11). The same shape backs
 * the Phase H bespoke working-tree viewer (`/api/git/diff` envelope). */
export interface CommitDiff {
  sha: string
  title: string
  files: FileDiff[]
}

/** `ActionLogEntry` — the node journal + the MUR refusal register (S8/U5):
 * `result` carries either `"ok"` or a `"rejected: …"` reason. */
export interface ActionLogEntry {
  timestamp: string
  action: string
  args: unknown
  result: string
}

/** The sealed context-pack (`handle_context_pack`) — hashed references only. */
export interface ContextPack {
  base_prompt: HashRef
  universal_prompt: HashRef
  handoff_prompt: HashRef
  specialized_prompt: HashRef | null
  agent_system: HashRef
  process_docs: HashRef[]
  authoring_knowledge: HashRef[]
  active_artifacts: HashRef[]
  runtime_context: Record<string, unknown>
  chat_history_authoritative: boolean
  notice: string
}

/** `AuditCommitResult` — `issues` is a list of MISSING things ("N manques"),
 * never a tick (U3/A9/V10 conformity card). */
export interface AuditCommit {
  rev: string
  title: string
  is_phase_commit: boolean
  ok: boolean
  issues: string[]
}

export interface LintDiagnostic {
  code: string
  message: string
  file: string | null
}
export interface Lint {
  ok: boolean
  errors: LintDiagnostic[]
  warnings: LintDiagnostic[]
}

/** A recorded terminal `.cast` (asciicast v2) available for replay (U6/V9). */
export interface TerminalCast {
  name: string
  path: string
  size_bytes: number
}
export interface TerminalSessions {
  sessions: TerminalCast[]
  claude_sessions: unknown[]
}

export interface ChatLog {
  id: string
  /** The session-sealed pack is REDUCED vs the full `/api/context-pack` one:
   * `handle_chat_session` (operator_server.rs) omits agent_system /
   * specialized_prompt / process_docs / active_artifacts. Typed `Partial` so a
   * consumer (D2 hash-drift) never assumes a field the backend did not seal. */
  context_pack: Partial<ContextPack>
  messages: { role: string; content: string; action?: string }[]
}

/** `GET /api/status` — the live process position (current sprint/phase). */
export function getStatus(signal?: AbortSignal): Promise<OperatorStatus> {
  return getJson<OperatorStatus>('/api/status', signal)
}

/** `GET /api/sprint-history` (active sprint) or `/{n}` for a specific one. */
export function getSprintHistory(sprint?: number, signal?: AbortSignal): Promise<SprintHistory> {
  const path = sprint == null ? '/api/sprint-history' : `/api/sprint-history/${sprint}`
  return getJson<SprintHistory>(path, signal)
}

export function getCommitDiff(sha: string, signal?: AbortSignal): Promise<CommitDiff> {
  return getJson<CommitDiff>(`/api/sprint-history/diff/${encodeURIComponent(sha)}`, signal)
}

export function getActionLog(signal?: AbortSignal): Promise<ActionLogEntry[]> {
  return getJson<ActionLogEntry[]>('/api/actions/log', signal)
}

export function postContextPack(req: {
  provider?: string
  intent?: string
  role?: string
  specialized_kind?: string
}): Promise<ContextPack> {
  return postJson<ContextPack>('/api/context-pack', req)
}

export function getAudit(rev: string, signal?: AbortSignal): Promise<AuditCommit> {
  return getJson<AuditCommit>(`/api/audit/${encodeURIComponent(rev)}`, signal)
}

export function getLint(signal?: AbortSignal): Promise<Lint> {
  return getJson<Lint>('/api/lint', signal)
}

export function getTerminalSessions(signal?: AbortSignal): Promise<TerminalSessions> {
  return getJson<TerminalSessions>('/api/terminal/sessions', signal)
}

/** Raw `.cast` text of a recorded session (path-validated server-side). */
export function getTerminalCast(name: string, signal?: AbortSignal): Promise<string> {
  return getText(`/api/terminal/sessions/${encodeURIComponent(name)}`, signal)
}

export function getChatLog(id: string, signal?: AbortSignal): Promise<ChatLog> {
  return getJson<ChatLog>(`/api/chat/${encodeURIComponent(id)}/log`, signal)
}

/** Same-origin WebSocket URL for the live PTY (`handle_terminal_ws`). The
 * HttpOnly cookie rides the handshake automatically — a WS cannot set a
 * custom auth header (kickoff Day-0 #5, why the cookie is the 1st gesture).
 * `resume` re-attaches a prior `claude --resume` session. */
export function terminalWsUrl(resume?: string): string {
  const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  const base = `${proto}//${window.location.host}/api/terminal/ws`
  return resume ? `${base}?resume=${encodeURIComponent(resume)}` : base
}

// --- Sprint 80 Phase H — VERIFY-plein (working-tree diff + live gates) ---
//
// Both shapes MIRROR a Rust struct read 1:1 (sprint_history.rs /
// gates.rs) and are RESTITUTED by the front, never computed. The
// diff-viewer renders the hunks the BACKEND parsed (`parse_unified_diff`),
// never a JS re-diff; the gates band restitutes each gate's distinct status,
// never an aggregated verdict (kickoff cardinal: 0 verdict calculé UI).

/** `GET /api/git/diff` — the repo working tree, computed in Rust
 * (`working_tree_diff_data`). `head` is the short HEAD sha (freshness /
 * `run@<rev>`); a partially staged file legitimately appears in BOTH
 * arrays (git semantics). `truncated` is set past `MAX_DIFF_LINES`
 * (20 000). `old_lineno`/`new_lineno` are nullable on each `DiffLine`. */
export interface WorkingTreeDiff {
  head: string
  unstaged: FileDiff[]
  staged: FileDiff[]
  truncated: boolean
}

/** The distinct, never-flattened status of a gate, mirroring the Rust
 * `GateStatus` enum (gates.rs:75-89, snake_case). EXACTLY these five — the
 * acceptance T2 words (PROVISIONAL / Not-evidenced / RIG-ABSENT) are NOT in
 * this enum and must never be fabricated from `/api/gates`. */
export type GateStatus = 'not_run' | 'not_applicable' | 'passed' | 'informational' | 'blocking'

/** One issue attached to a gate (gates.rs `GateIssueView`). `line` is
 * ALWAYS `null` in S80 (no line anchor before the `GateResult.issues ->
 * struct` refactor, carry S81); `file`, when present, is a `.planning/`
 * basename (the lint-planning source), NOT a change-set path — so V5/V6
 * (per-line gutter / per-change-set-file gate marker) are degraded to S81. */
export interface GateIssueView {
  message: string
  file: string | null
  line: number | null
}

/** One gate's restituted state (gates.rs `GateEntryView`). A single `gate`
 * name can appear under more than one `status` (lint-planning splits its
 * errors → `blocking` and warnings → `informational`), so consumers key by
 * `(gate, status)`, never by `gate` alone. */
export interface GateEntryView {
  gate: string
  status: GateStatus
  issues: GateIssueView[]
}

/** `GET /api/gates` — a flat list of restituted gate states with NO
 * aggregate field at the root (gates.rs:111-121: no `overall`/`all_passed`/
 * score). The Operator closes no verdict; the front restitutes 1:1. */
export interface GatesView {
  gates: GateEntryView[]
}

/** The working-tree diff the Phase H VERIFY diff-viewer renders. */
export function getWorkingTreeDiff(signal?: AbortSignal): Promise<WorkingTreeDiff> {
  return getJson<WorkingTreeDiff>('/api/git/diff', signal)
}

/** The live gate registry the Phase H VERIFY gates band restitutes. */
export function getGates(signal?: AbortSignal): Promise<GatesView> {
  return getJson<GatesView>('/api/gates', signal)
}
