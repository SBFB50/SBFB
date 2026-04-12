// SPDX-License-Identifier: AGPL-3.0-or-later
import type { TabBlockKV } from "../schema";

export function KVBlock({ block }: { block: TabBlockKV }) {
  return (
    <dl className="grid grid-cols-1 gap-2 text-xs sm:grid-cols-2">
      {block.items.map((item, i) => (
        <div
          key={i}
          className="flex flex-col rounded-md border border-border bg-background/60 px-3 py-2"
        >
          <dt className="text-[10px] uppercase tracking-wider text-muted-foreground">
            {item.label}
          </dt>
          <dd className="font-mono text-sm">{item.value}</dd>
          {item.hint && (
            <dd className="text-[10px] italic text-muted-foreground">
              {item.hint}
            </dd>
          )}
        </div>
      ))}
    </dl>
  );
}
