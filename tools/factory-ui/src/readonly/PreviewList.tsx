// SPDX-License-Identifier: AGPL-3.0-or-later

import type { AppEntry } from "./types";

export function PreviewList({
  apps,
  onSelect,
}: {
  apps: AppEntry[];
  onSelect?: (name: string) => void;
}) {
  if (apps.length === 0) {
    return (
      <p className="py-8 text-center text-sm text-[#8b949e]">
        Aucune app trouvée sur le réseau.
      </p>
    );
  }

  return (
    <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
      {apps.map((app) => (
        <button
          key={app.name}
          type="button"
          onClick={() => onSelect?.(app.name)}
          className="rounded-lg border border-[#30363d] bg-[#1c2128] p-4 text-left transition-colors hover:border-[#58a6ff]/50"
        >
          <h3 className="mb-1 text-sm font-semibold text-white">{app.name}</h3>
          <p className="mb-2 text-xs text-[#8b949e]">{app.description}</p>
          <div className="flex items-center gap-2 text-xs">
            <span className="font-mono text-[#8b949e]">v{app.version}</span>
            <span className="rounded bg-[#161b22] px-1.5 py-0.5 text-[#8b949e]">
              {app.category}
            </span>
            <span
              className={app.published ? "text-[#3fb950]" : "text-[#d29922]"}
            >
              {app.published ? "● Publiée" : "○ Dev"}
            </span>
          </div>
        </button>
      ))}
    </div>
  );
}
