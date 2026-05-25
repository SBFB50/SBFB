// SPDX-License-Identifier: AGPL-3.0-or-later

export interface ChangelogEntry {
  version: string;
  date: string;
  changes: string[];
}

export function ChangelogPanel({ entries }: { entries: ChangelogEntry[] }) {
  if (entries.length === 0) {
    return (
      <p className="py-4 text-sm text-[#8b949e]">
        Aucun changelog disponible.
      </p>
    );
  }

  return (
    <div className="space-y-4">
      {entries.map((entry) => (
        <div key={entry.version} className="border-l-2 border-[#30363d] pl-4">
          <div className="flex items-baseline gap-2">
            <span className="font-mono text-sm font-semibold text-[#58a6ff]">
              v{entry.version}
            </span>
            <span className="text-xs text-[#8b949e]">{entry.date}</span>
          </div>
          <ul className="mt-1 space-y-0.5">
            {entry.changes.map((change, i) => (
              <li key={i} className="text-xs text-[#c9d1d9]">
                {change}
              </li>
            ))}
          </ul>
        </div>
      ))}
    </div>
  );
}
