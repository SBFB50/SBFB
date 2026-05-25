// SPDX-License-Identifier: AGPL-3.0-or-later

import { useApi } from "@/hooks/useApi";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { useState } from "react";

interface FileChange {
  path: string;
  insertions: number;
  deletions: number;
  status: string;
}

interface Finding {
  severity: string;
  code: string;
  description: string;
  status: string;
}

interface PhaseHistory {
  letter: string;
  title: string;
  commit_sha: string | null;
  commit_date: string | null;
  commit_type: string | null;
  preflight_verdict: string | null;
  review_verdict: string | null;
  codex_confirmed: number | null;
  codex_partial: number | null;
  codex_gap: number | null;
  rust_delta: number;
  vitest_delta: number;
  files_changed: FileChange[];
  deliverables: string[];
  findings: Finding[];
}

interface CommitInfo {
  sha: string;
  short: string;
  title: string;
  author: string;
  date: string;
  commit_type: string;
  is_phase: boolean;
  phase: string | null;
  insertions: number;
  deletions: number;
  files: string[];
  body_sections: string[];
}

interface PhaseTestDelta {
  phase: string;
  rust_delta: number;
  vitest_delta: number;
  detail: string;
}

interface ScopeCutItem {
  number: number;
  item: string;
  target: string;
  respected: boolean;
}

interface CarryItem {
  code: string;
  description: string;
  disposition: string;
  phase_closed: string | null;
}

interface VerificationCheck {
  number: number;
  name: string;
  command: string;
  result: string;
}

interface PreflightPhase {
  phase: string;
  verdict: string;
  file: string;
}

interface SprintHistoryData {
  sprint: number;
  status: string;
  branch: string;
  head: string;
  entry_tip: string | null;
  exit_tip: string | null;
  total_commits: number;
  phase_commits: number;
  chore_commits: number;
  phases: PhaseHistory[];
  commits: CommitInfo[];
  tests: {
    rust_entry: number;
    rust_exit: number;
    rust_delta: number;
    vitest_entry: number;
    vitest_exit: number;
    vitest_delta: number;
    size_limit: string;
    per_phase: PhaseTestDelta[];
  };
  scope_cuts: ScopeCutItem[];
  carries_closed: CarryItem[];
  carries_open: CarryItem[];
  verification: {
    total_checks: number;
    passed: number;
    failed: number;
    checks: VerificationCheck[];
  } | null;
  preflight_bilan: {
    total: number;
    execute: number;
    plan_adapt: number;
    design_conflict: number;
    phases: PreflightPhase[];
  };
}

function VerdictBadge({ verdict }: { verdict: string | null }) {
  if (!verdict) return <Badge variant="outline">-</Badge>;
  const colors: Record<string, string> = {
    PASS: "bg-green-600/20 text-green-400 border-green-600/40",
    EXECUTE: "bg-green-600/20 text-green-400 border-green-600/40",
    "PASS-PENDING": "bg-yellow-600/20 text-yellow-400 border-yellow-600/40",
    "PLAN-ADAPT": "bg-yellow-600/20 text-yellow-400 border-yellow-600/40",
    FAIL: "bg-red-600/20 text-red-400 border-red-600/40",
    "DESIGN-CONFLICT": "bg-red-600/20 text-red-400 border-red-600/40",
  };
  return (
    <Badge className={colors[verdict] ?? "bg-zinc-600/20 text-zinc-400"}>
      {verdict}
    </Badge>
  );
}

function SeverityBadge({ severity }: { severity: string }) {
  const colors: Record<string, string> = {
    P0: "bg-red-600/20 text-red-400",
    P1: "bg-orange-600/20 text-orange-400",
    P2: "bg-yellow-600/20 text-yellow-400",
    P3: "bg-zinc-600/20 text-zinc-400",
  };
  return <Badge className={colors[severity] ?? ""}>{severity}</Badge>;
}

function PhaseCard({ phase }: { phase: PhaseHistory }) {
  const [expanded, setExpanded] = useState(false);
  const date = phase.commit_date
    ? new Date(phase.commit_date).toLocaleDateString("fr-FR", {
        day: "2-digit",
        month: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
      })
    : "-";

  return (
    <Card className="bg-zinc-900/50 border-zinc-700/50">
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-base flex items-center gap-2">
            <span className="text-lg font-bold text-blue-400">
              {phase.letter}
            </span>
            <span className="text-zinc-300">{phase.title}</span>
          </CardTitle>
          <div className="flex items-center gap-2">
            <VerdictBadge verdict={phase.review_verdict} />
            {phase.commit_sha && (
              <code className="text-xs text-zinc-500">{phase.commit_sha}</code>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent className="pt-0">
        <div className="grid grid-cols-2 md:grid-cols-5 gap-3 text-sm">
          <div>
            <span className="text-zinc-500">Date</span>
            <div className="text-zinc-300">{date}</div>
          </div>
          <div>
            <span className="text-zinc-500">Type</span>
            <div>
              <Badge variant="outline">{phase.commit_type ?? "-"}</Badge>
            </div>
          </div>
          <div>
            <span className="text-zinc-500">Preflight</span>
            <div>
              <VerdictBadge verdict={phase.preflight_verdict} />
            </div>
          </div>
          <div>
            <span className="text-zinc-500">Tests</span>
            <div className="text-zinc-300">
              {phase.rust_delta > 0 ? `+${phase.rust_delta} Rust` : "docs-only"}
            </div>
          </div>
          <div>
            <span className="text-zinc-500">Codex</span>
            <div className="text-zinc-300 text-xs">
              {phase.codex_confirmed != null
                ? `${phase.codex_confirmed}C ${phase.codex_partial ?? 0}P ${phase.codex_gap ?? 0}G`
                : "-"}
            </div>
          </div>
        </div>

        {phase.findings.length > 0 && (
          <div className="mt-3">
            <button
              onClick={() => setExpanded(!expanded)}
              className="text-xs text-zinc-400 hover:text-zinc-200"
            >
              {expanded ? "Masquer" : "Voir"} {phase.findings.length} findings
            </button>
            {expanded && (
              <div className="mt-2 space-y-1">
                {phase.findings.map((f, i) => (
                  <div key={i} className="flex items-start gap-2 text-xs">
                    <SeverityBadge severity={f.severity} />
                    <span className="text-zinc-400">{f.code}</span>
                    <span
                      className={
                        f.status === "resolved"
                          ? "text-green-400 line-through"
                          : "text-zinc-300"
                      }
                    >
                      {f.description.slice(0, 120)}
                      {f.description.length > 120 ? "..." : ""}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {expanded && phase.files_changed.length > 0 && (
          <div className="mt-3">
            <span className="text-xs text-zinc-500">
              {phase.files_changed.length} fichiers
            </span>
            <div className="mt-1 max-h-40 overflow-auto text-xs">
              {phase.files_changed.map((f, i) => (
                <div key={i} className="flex gap-2 text-zinc-400">
                  <span className="text-green-400">+{f.insertions}</span>
                  <span className="text-red-400">-{f.deletions}</span>
                  <span className="truncate">{f.path}</span>
                </div>
              ))}
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

export function SprintHistory() {
  const { data, loading, error } = useApi<SprintHistoryData>("/sprint-history");

  if (loading) return <div className="text-zinc-400">Chargement...</div>;
  if (error) return <div className="text-red-400">Erreur : {error}</div>;
  if (!data) return <div className="text-zinc-400">Aucun sprint actif</div>;

  const totalInsertions = data.commits.reduce((s, c) => s + c.insertions, 0);
  const totalDeletions = data.commits.reduce((s, c) => s + c.deletions, 0);

  return (
    <div className="space-y-6 max-w-5xl">
      <div>
        <h1 className="text-2xl font-bold text-zinc-100">
          Sprint {data.sprint}
        </h1>
        <p className="text-zinc-400 text-sm mt-1">
          {data.status === "completed" ? "Termine" : "En cours"} — {data.branch}{" "}
          @ <code>{data.head}</code> — {data.total_commits} commits (
          {data.phase_commits} phases, {data.chore_commits} chore)
        </p>
      </div>

      {/* Stats cards */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Card className="bg-zinc-900/50 border-zinc-700/50">
          <CardContent className="pt-4">
            <div className="text-2xl font-bold text-blue-400">
              {data.phases.length}
            </div>
            <div className="text-xs text-zinc-500">Phases</div>
          </CardContent>
        </Card>
        <Card className="bg-zinc-900/50 border-zinc-700/50">
          <CardContent className="pt-4">
            <div className="text-2xl font-bold text-green-400">
              +{data.tests.rust_delta}
            </div>
            <div className="text-xs text-zinc-500">
              Tests Rust ({data.tests.rust_entry} → {data.tests.rust_exit})
            </div>
          </CardContent>
        </Card>
        <Card className="bg-zinc-900/50 border-zinc-700/50">
          <CardContent className="pt-4">
            <div className="text-2xl font-bold text-emerald-400">
              +{totalInsertions.toLocaleString()}
            </div>
            <div className="text-xs text-zinc-500">
              Insertions / -{totalDeletions.toLocaleString()} deletions
            </div>
          </CardContent>
        </Card>
        <Card className="bg-zinc-900/50 border-zinc-700/50">
          <CardContent className="pt-4">
            <div className="text-2xl font-bold text-purple-400">
              {data.preflight_bilan.execute}/{data.preflight_bilan.total}
            </div>
            <div className="text-xs text-zinc-500">G8 EXECUTE</div>
          </CardContent>
        </Card>
      </div>

      {/* Phase timeline */}
      <div>
        <h2 className="text-lg font-semibold text-zinc-200 mb-3">
          Phases
        </h2>
        <div className="space-y-3">
          {data.phases.map((phase) => (
            <PhaseCard key={phase.letter} phase={phase} />
          ))}
        </div>
      </div>

      <Separator className="bg-zinc-700/50" />

      {/* Verification */}
      {data.verification && (
        <div>
          <h2 className="text-lg font-semibold text-zinc-200 mb-3">
            Verification ({data.verification.passed}/{data.verification.total_checks})
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-1 text-xs">
            {data.verification.checks.map((check) => (
              <div
                key={check.number}
                className="flex items-center gap-2 py-1 px-2 rounded bg-zinc-900/30"
              >
                <span
                  className={
                    check.result.includes("PASS")
                      ? "text-green-400"
                      : "text-red-400"
                  }
                >
                  {check.result.includes("PASS") ? "PASS" : "FAIL"}
                </span>
                <span className="text-zinc-400">#{check.number}</span>
                <span className="text-zinc-300 truncate">{check.name}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      <Separator className="bg-zinc-700/50" />

      {/* Scope cuts */}
      <div>
        <h2 className="text-lg font-semibold text-zinc-200 mb-3">
          Scope Cuts ({data.scope_cuts.filter((s) => s.respected).length}/
          {data.scope_cuts.length})
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-1 text-xs">
          {data.scope_cuts.map((sc) => (
            <div
              key={sc.number}
              className="flex items-center gap-2 py-1 px-2 rounded bg-zinc-900/30"
            >
              <span className={sc.respected ? "text-green-400" : "text-red-400"}>
                {sc.respected ? "OUI" : "NON"}
              </span>
              <span className="text-zinc-300 truncate">{sc.item}</span>
              <span className="text-zinc-500 ml-auto">{sc.target}</span>
            </div>
          ))}
        </div>
      </div>

      <Separator className="bg-zinc-700/50" />

      {/* Carries */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div>
          <h2 className="text-lg font-semibold text-green-400 mb-3">
            Carries CLOSED ({data.carries_closed.length})
          </h2>
          <div className="space-y-1 text-xs">
            {data.carries_closed.map((c, i) => (
              <div key={i} className="py-1 px-2 rounded bg-zinc-900/30">
                <span className="text-green-400 font-mono">{c.code}</span>
                <span className="text-zinc-500 ml-2">{c.phase_closed}</span>
              </div>
            ))}
          </div>
        </div>
        <div>
          <h2 className="text-lg font-semibold text-yellow-400 mb-3">
            Carries OPEN ({data.carries_open.length})
          </h2>
          <div className="space-y-1 text-xs">
            {data.carries_open.map((c, i) => (
              <div key={i} className="py-1 px-2 rounded bg-zinc-900/30">
                <span className="text-yellow-400 font-mono">{c.code}</span>
                <span className="text-zinc-400 ml-2">{c.description}</span>
              </div>
            ))}
          </div>
        </div>
      </div>

      <Separator className="bg-zinc-700/50" />

      {/* Commits */}
      <div>
        <h2 className="text-lg font-semibold text-zinc-200 mb-3">
          Commits ({data.commits.length})
        </h2>
        <div className="space-y-1 text-xs">
          {data.commits.map((commit) => (
            <div
              key={commit.sha}
              className="flex items-center gap-2 py-1 px-2 rounded bg-zinc-900/30"
            >
              <code className="text-zinc-500 w-16 shrink-0">
                {commit.short}
              </code>
              {commit.is_phase ? (
                <Badge className="bg-blue-600/20 text-blue-400 w-6 justify-center">
                  {commit.phase}
                </Badge>
              ) : (
                <Badge variant="outline" className="w-6 justify-center text-zinc-500">
                  c
                </Badge>
              )}
              <span className="text-zinc-300 truncate flex-1">
                {commit.title}
              </span>
              <span className="text-green-400">+{commit.insertions}</span>
              <span className="text-red-400">-{commit.deletions}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
