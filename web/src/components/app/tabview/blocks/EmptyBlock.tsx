import type { TabBlockEmpty } from "../schema";

export function EmptyBlock({ block }: { block: TabBlockEmpty }) {
  return (
    <div className="rounded-md border border-dashed border-border bg-background/30 px-4 py-6 text-center">
      <p className="text-xs italic text-muted-foreground">{block.text}</p>
    </div>
  );
}
