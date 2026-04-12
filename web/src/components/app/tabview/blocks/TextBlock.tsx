// SPDX-License-Identifier: AGPL-3.0-or-later
import type { TabBlockText } from "../schema";
import { cn } from "@/lib/utils";

export function TextBlock({ block }: { block: TabBlockText }) {
  return (
    <p
      className={cn(
        "text-sm leading-relaxed",
        block.muted && "text-muted-foreground",
      )}
    >
      {block.text}
    </p>
  );
}
