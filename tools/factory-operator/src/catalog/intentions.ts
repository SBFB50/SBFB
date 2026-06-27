// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Sprint 80 Phase C — the intention library (S1 fold) and the execution
// provider set, as BUILD-TIME assets imported statically (plan-adaptation
// #2). The plan's "via POST /api/artifacts/draft + lecture front" is doubly
// blocked: '.planning/factory/' is absent from ARTIFACT_DRAFT_ALLOWLIST
// (operator_server.rs:28-35) and the Operator's ServeDir is rooted at
// tools/factory-operator/bundle, not the repo root (:47) — a runtime fetch
// of /.planning/factory/intentions.json 404s. A static import keeps it
// "neuf-léger côté front, 0 route". Versioned richer in S81.

/**
 * A composer preset. The intention CARRIES the human CTA (`label`, sans);
 * the technical `kind` (the prompt-adaptation kind, folded under "détails
 * techniques") stays under the hood — intentions-pas-jargon. `kind` must be
 * a real PROMPT_KIND so the inspector `GET /api/prompt/{kind}` resolves.
 */
export interface IntentionPreset {
  id: string
  label: string
  hint: string
  kind: string
}

export const INTENTIONS: readonly IntentionPreset[] = [
  {
    id: 'prepare',
    label: 'Préparer la phase',
    hint: 'décrivez ce que la phase doit livrer…',
    kind: 'preflight',
  },
  {
    id: 'verify',
    label: 'Vérifier avant validation',
    hint: 'décrivez ce qui doit être vérifié…',
    kind: 'phase-review',
  },
  {
    id: 'handoff',
    label: 'Transmettre à un autre agent',
    hint: 'décrivez le contexte à transmettre…',
    kind: 'handoff',
  },
] as const

/**
 * The chat EXECUTION provider axis — a closed set mirroring the Rust
 * `ExecutionTarget::from_provider` arms (provider_router.rs:80-93):
 * 'local' aliases Ollama (UX intention "run locally"), 'network' is the P2P
 * arm, 'claude' (and anything unknown) is the default pilot. This is a
 * DISTINCT axis from `GET /api/providers` (the prompt-adaptation set
 * {claude, codex, gpt, local, human}); the chat session/send/stream route
 * on THIS axis, so the composer attribute offers exactly these three.
 */
export interface ExecProviderOption {
  id: ExecProvider
  label: string
  note: string
}

export type ExecProvider = 'claude' | 'local' | 'network'

export const EXEC_PROVIDERS: readonly ExecProviderOption[] = [
  { id: 'claude', label: 'Claude', note: 'cloud' },
  { id: 'local', label: 'Local', note: 'Ollama' },
  { id: 'network', label: 'Réseau', note: 'P2P' },
] as const

export const DEFAULT_PROVIDER: ExecProvider = 'claude'
