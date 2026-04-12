// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * TableBlock — renders a `table` TabView block.
 *
 * Sprint 8 Phase E polish: click-to-sort column headers. The
 * sort cycles through asc → desc → none and lives entirely in
 * component state, so the TabView Zod schema (and its Python
 * snapshot) stays untouched. Numeric columns are sorted
 * numerically; string columns fall back to `localeCompare`
 * which handles accented characters correctly in the
 * francophone gov tabs.
 *
 * The header button has `aria-sort` so screen readers pick up
 * the current direction, and the visible arrow mirrors the
 * state for sighted users. Clicking the active column again
 * flips direction; clicking a third time resets to the
 * server-provided row order (no sort).
 */

import { useMemo, useState } from "react";
import { ArrowDown, ArrowUp, ArrowUpDown } from "lucide-react";
import type { TabBlockTable, TableRow } from "../schema";
import { cn } from "@/lib/utils";

type SortDirection = "asc" | "desc";
interface SortState {
  key: string;
  direction: SortDirection;
}

function compareValues(a: unknown, b: unknown): number {
  // Treat null/undefined as "greater than" so they sink to the
  // bottom on ascending sort — the gov tabs use `—` for missing
  // cells and the user's mental model is "missing rows last".
  const aNull = a === null || a === undefined;
  const bNull = b === null || b === undefined;
  if (aNull && bNull) return 0;
  if (aNull) return 1;
  if (bNull) return -1;

  if (typeof a === "number" && typeof b === "number") {
    return a - b;
  }
  const aStr = String(a);
  const bStr = String(b);
  return aStr.localeCompare(bStr, "fr", { numeric: true, sensitivity: "base" });
}

export function TableBlock({ block }: { block: TabBlockTable }) {
  const [sort, setSort] = useState<SortState | null>(null);

  const sortedRows = useMemo<TableRow[]>(() => {
    if (!sort) return block.rows;
    // `slice()` keeps the original row array stable so React
    // Query cache consumers don't see their data mutated out
    // from under them.
    const copy = block.rows.slice();
    copy.sort((a, b) => {
      const cmp = compareValues(a[sort.key], b[sort.key]);
      return sort.direction === "asc" ? cmp : -cmp;
    });
    return copy;
  }, [block.rows, sort]);

  const cycleSort = (key: string) => {
    setSort((prev) => {
      if (!prev || prev.key !== key) return { key, direction: "asc" };
      if (prev.direction === "asc") return { key, direction: "desc" };
      return null;
    });
  };

  if (block.rows.length === 0) {
    return (
      <p className="text-xs italic text-muted-foreground">
        {block.empty_text ?? "(aucune ligne)"}
      </p>
    );
  }

  return (
    <div className="overflow-x-auto rounded-md border border-border">
      <table className="min-w-full text-xs">
        <thead className="bg-muted/40">
          <tr>
            {block.columns.map((col) => {
              const isActive = sort?.key === col.key;
              const direction = isActive ? sort?.direction : undefined;
              const ariaSort: "ascending" | "descending" | "none" = isActive
                ? direction === "asc"
                  ? "ascending"
                  : "descending"
                : "none";
              return (
                <th
                  key={col.key}
                  aria-sort={ariaSort}
                  className={cn(
                    "px-3 py-2 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground",
                    col.align === "right" && "text-right",
                    col.align === "center" && "text-center",
                    col.align === "left" && "text-left",
                  )}
                >
                  <button
                    type="button"
                    onClick={() => cycleSort(col.key)}
                    className={cn(
                      "inline-flex items-center gap-1 text-[10px] font-semibold uppercase tracking-wider transition-colors hover:text-foreground focus:outline-none focus-visible:text-foreground",
                      isActive && "text-foreground",
                    )}
                    data-testid={`tableblock-sort-${col.key}`}
                  >
                    <span>{col.label}</span>
                    {isActive ? (
                      direction === "asc" ? (
                        <ArrowUp className="size-3" />
                      ) : (
                        <ArrowDown className="size-3" />
                      )
                    ) : (
                      <ArrowUpDown className="size-3 opacity-40" />
                    )}
                  </button>
                </th>
              );
            })}
          </tr>
        </thead>
        <tbody>
          {sortedRows.map((row, i) => (
            <tr
              key={i}
              className="border-t border-border/60 odd:bg-background/40 even:bg-background/20"
            >
              {block.columns.map((col) => {
                const raw = row[col.key];
                const display = raw == null ? "—" : String(raw);
                return (
                  <td
                    key={col.key}
                    className={cn(
                      "px-3 py-2 font-mono",
                      col.align === "right" && "text-right",
                      col.align === "center" && "text-center",
                      raw == null && "text-muted-foreground",
                    )}
                  >
                    {display}
                  </td>
                );
              })}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
