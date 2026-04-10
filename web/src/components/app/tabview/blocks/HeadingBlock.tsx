import type { TabBlockHeading } from "../schema";

const LEVEL_CLS: Record<1 | 2 | 3, string> = {
  1: "text-xl font-bold tracking-tight",
  2: "text-base font-semibold tracking-tight",
  3: "text-sm font-semibold uppercase tracking-wider text-muted-foreground",
};

export function HeadingBlock({ block }: { block: TabBlockHeading }) {
  const Tag = (`h${block.level + 1}` as "h2" | "h3" | "h4");
  return <Tag className={LEVEL_CLS[block.level]}>{block.text}</Tag>;
}
