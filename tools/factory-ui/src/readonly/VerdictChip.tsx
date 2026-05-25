// SPDX-License-Identifier: AGPL-3.0-or-later

import type { Verdict } from "./types";
import { VERDICT_LABELS } from "./labels";

const VERDICT_COLORS: Record<Verdict, string> = {
  PASS: "bg-green-900/40 text-green-400 border-green-700/50",
  FAIL: "bg-red-900/40 text-red-400 border-red-700/50",
  CONCERN: "bg-yellow-900/40 text-yellow-400 border-yellow-700/50",
  PENDING: "bg-zinc-800/40 text-zinc-400 border-zinc-600/50",
};

const VERDICT_ICONS: Record<Verdict, string> = {
  PASS: "✓",
  FAIL: "✗",
  CONCERN: "⚠",
  PENDING: "…",
};

export function VerdictChip({ verdict }: { verdict: Verdict }) {
  return (
    <span
      className={`inline-flex items-center gap-1 rounded border px-2 py-0.5 text-xs font-medium ${VERDICT_COLORS[verdict]}`}
    >
      <span>{VERDICT_ICONS[verdict]}</span>
      {VERDICT_LABELS[verdict]}
    </span>
  );
}
