// SPDX-License-Identifier: AGPL-3.0-or-later
import type { BlockTone, TabBlockMetric } from "../schema";
import { cn } from "@/lib/utils";

const TONE_TEXT: Record<BlockTone, string> = {
  neutral: "text-foreground",
  ok: "text-emerald-400",
  warn: "text-amber-400",
  danger: "text-destructive",
};

export function MetricBlock({ block }: { block: TabBlockMetric }) {
  const deltaSign = block.delta == null ? null : block.delta > 0 ? "+" : "";
  return (
    <div className="rounded-lg border border-border bg-background/60 px-4 py-3">
      <div className="text-[10px] uppercase tracking-wider text-muted-foreground">
        {block.label}
      </div>
      <div className="mt-1 flex items-baseline gap-2">
        <span className={cn("text-2xl font-semibold", TONE_TEXT[block.tone])}>
          {block.value}
        </span>
        {block.unit && (
          <span className="text-xs text-muted-foreground">{block.unit}</span>
        )}
        {block.delta != null && (
          <span
            className={cn(
              "text-xs font-medium",
              block.delta > 0 && "text-emerald-400",
              block.delta < 0 && "text-destructive",
            )}
          >
            {deltaSign}
            {block.delta}
          </span>
        )}
      </div>
    </div>
  );
}
