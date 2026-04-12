// SPDX-License-Identifier: AGPL-3.0-or-later
import type { TabBlockSection } from "../schema";
import { TabBlockRenderer } from "../TabBlockRenderer";

export function SectionBlock({ block }: { block: TabBlockSection }) {
  return (
    <section className="rounded-lg border border-border bg-muted/20 p-4">
      {block.title && (
        <h3 className="mb-3 text-sm font-semibold uppercase tracking-wider text-muted-foreground">
          {block.title}
        </h3>
      )}
      {block.blocks.length === 0 ? (
        <p className="text-xs italic text-muted-foreground">(section vide)</p>
      ) : (
        <div className="space-y-3">
          {block.blocks.map((b, i) => (
            <TabBlockRenderer key={i} block={b} />
          ))}
        </div>
      )}
    </section>
  );
}
