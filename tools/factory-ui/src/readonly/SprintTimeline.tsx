// SPDX-License-Identifier: AGPL-3.0-or-later

import type { PhaseInfo } from "./types";
import { StatusBadge } from "./StatusBadge";

export function SprintTimeline({ phases }: { phases: PhaseInfo[] }) {
  return (
    <div className="grid grid-cols-2 gap-2 sm:grid-cols-3 md:grid-cols-4">
      {phases.map((phase) => (
        <div
          key={phase.id}
          className="flex items-center justify-between rounded border border-[#30363d] bg-[#1c2128] px-3 py-2"
        >
          <span className="text-sm font-medium text-white">{phase.label}</span>
          <StatusBadge status={phase.status} />
        </div>
      ))}
    </div>
  );
}
