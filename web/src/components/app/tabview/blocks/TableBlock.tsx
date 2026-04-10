import type { TabBlockTable } from "../schema";
import { cn } from "@/lib/utils";

export function TableBlock({ block }: { block: TabBlockTable }) {
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
            {block.columns.map((col) => (
              <th
                key={col.key}
                className={cn(
                  "px-3 py-2 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground",
                  col.align === "right" && "text-right",
                  col.align === "center" && "text-center",
                  col.align === "left" && "text-left",
                )}
              >
                {col.label}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {block.rows.map((row, i) => (
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
