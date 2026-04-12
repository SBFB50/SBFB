// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 6 Phase B — schema-driven tab renderer.
 *
 * Consumes a `TabView` validated by `parseTabView` and walks the
 * block tree via `<TabBlockRenderer>`. Each block kind gets a
 * small, self-contained component in `./blocks/*`. No app-specific
 * code. No dynamic imports.
 *
 * Charts (`chart_line`, `chart_bar`) are SVG-inline to keep the
 * bundle lean — no recharts / d3 / victory — and match the
 * Sprint 5 Day 0 decision to drop all legacy chart deps.
 */

import type { TabView } from "./schema";
import { TabBlockRenderer } from "./TabBlockRenderer";

export function TabViewRenderer({ tabView }: { tabView: TabView }) {
  return (
    <div className="space-y-4">
      {tabView.title && (
        <h2 className="text-lg font-semibold tracking-tight">
          {tabView.title}
        </h2>
      )}
      {tabView.blocks.length === 0 ? (
        <p className="text-xs italic text-muted-foreground">
          (aucun bloc à afficher)
        </p>
      ) : (
        <div className="space-y-3">
          {tabView.blocks.map((block, i) => (
            <TabBlockRenderer key={i} block={block} />
          ))}
        </div>
      )}
    </div>
  );
}
