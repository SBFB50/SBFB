// SPDX-License-Identifier: AGPL-3.0-or-later

import type { PhaseStatus, Verdict } from "./types";

export const PHASE_STATUS_LABELS: Record<PhaseStatus, string> = {
  done: "Terminée",
  active: "En cours",
  pending: "En attente",
  error: "Erreur",
};

export const VERDICT_LABELS: Record<Verdict, string> = {
  PASS: "Validé",
  FAIL: "Échoué",
  CONCERN: "Attention",
  PENDING: "En attente",
};

export const PHASE_KIND_LABELS: Record<string, string> = {
  preflight: "Préparer la phase",
  "phase-review": "Relire la phase",
  "phase-auditor": "Vérifier avant validation",
  "commit-body": "Préparer le message de commit",
  handoff: "Transmettre à un autre agent",
  "audit-gate": "Auditer le sprint",
};

export const PHASE_KIND_DESCRIPTIONS: Record<string, string> = {
  preflight: "Lancer le preflight avant de coder",
  "phase-review": "Review après implémentation",
  "phase-auditor": "Audit indépendant de la phase",
  "commit-body": "Générer le body du commit",
  handoff: "Handoff vers Claude/Codex/GPT",
  "audit-gate": "Audit gate complet du sprint",
};

export const PROVIDER_LABELS: Record<string, string> = {
  claude: "Claude",
  codex: "Codex",
  gpt: "GPT",
  local: "Agent local/offline",
  human: "Humain",
};
