// SPDX-License-Identifier: AGPL-3.0-or-later

import type { PhaseStatus } from "./types";
import { PHASE_STATUS_LABELS } from "./labels";

const STATUS_COLORS: Record<PhaseStatus, string> = {
  done: "bg-green-900/40 text-green-400 border-green-700/50",
  active: "bg-blue-900/40 text-blue-400 border-blue-700/50",
  pending: "bg-zinc-800/40 text-zinc-400 border-zinc-600/50",
  error: "bg-red-900/40 text-red-400 border-red-700/50",
};

const DOT_COLORS: Record<PhaseStatus, string> = {
  done: "bg-green-400",
  active: "bg-blue-400 animate-pulse",
  pending: "bg-zinc-500",
  error: "bg-red-400",
};

export function StatusBadge({ status }: { status: PhaseStatus }) {
  return (
    <span
      className={`inline-flex items-center gap-1.5 rounded-full border px-2.5 py-0.5 text-xs font-medium ${STATUS_COLORS[status]}`}
    >
      <span className={`h-1.5 w-1.5 rounded-full ${DOT_COLORS[status]}`} />
      {PHASE_STATUS_LABELS[status]}
    </span>
  );
}
