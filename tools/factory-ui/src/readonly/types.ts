// SPDX-License-Identifier: AGPL-3.0-or-later

export type PhaseStatus = "done" | "active" | "pending" | "error";

export type Verdict = "PASS" | "FAIL" | "CONCERN" | "PENDING";

export interface PhaseInfo {
  id: string;
  label: string;
  status: PhaseStatus;
  artifacts: {
    preflight: boolean;
    review: boolean;
    codex: boolean;
  };
}

export interface SprintStatus {
  sprint: number;
  title: string;
  phases: PhaseInfo[];
  test_counts: {
    rust: number;
    vitest: number;
    size_limit: number;
  };
  head: string;
}

export interface ProofCardData {
  project_id: string;
  name: string;
  version: string;
  commit_source: string;
  archive_hash: string;
  signer_pubkey: string;
  verified: boolean;
}

export interface AppEntry {
  name: string;
  version: string;
  category: string;
  description: string;
  published: boolean;
}

export interface LintResult {
  level: "warning" | "error";
  message: string;
  file: string;
  line?: number;
}

export interface AuditResult {
  rev: string;
  sections_present: string[];
  sections_missing: string[];
  review_check: boolean;
  codex_check: boolean;
}

export interface ProviderInfo {
  id: string;
  label: string;
  kind: string;
}
