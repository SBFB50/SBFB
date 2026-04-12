// SPDX-License-Identifier: AGPL-3.0-or-later
import type { BlockTone, TabBlockBadgeList } from "../schema";
import { Badge } from "@/components/ui/badge";

const TONE_VARIANT: Record<
  BlockTone,
  "default" | "secondary" | "destructive" | "outline"
> = {
  neutral: "outline",
  ok: "secondary",
  warn: "default",
  danger: "destructive",
};

export function BadgeListBlock({ block }: { block: TabBlockBadgeList }) {
  return (
    <div className="flex flex-wrap gap-2">
      {block.items.map((item, i) => (
        <Badge key={i} variant={TONE_VARIANT[item.tone]}>
          {item.label}
        </Badge>
      ))}
    </div>
  );
}
