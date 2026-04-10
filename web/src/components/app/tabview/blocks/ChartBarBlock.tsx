import type { BlockTone, TabBlockChartBar } from "../schema";

const W = 400;
const H = 120;
const PAD_X = 32;
const PAD_Y = 24;

const TONE_FILL: Record<BlockTone, string> = {
  neutral: "fill-primary/70",
  ok: "fill-emerald-500/80",
  warn: "fill-amber-500/80",
  danger: "fill-destructive/80",
};

export function ChartBarBlock({ block }: { block: TabBlockChartBar }) {
  if (block.bars.length === 0) {
    return (
      <div className="rounded-lg border border-border bg-background/60 p-3 text-xs italic text-muted-foreground">
        {block.label} — (aucune barre)
      </div>
    );
  }
  const values = block.bars.map((b) => b.value);
  const vMax = Math.max(...values, 0);
  const vRange = vMax || 1;
  const innerW = W - PAD_X * 2;
  const barGap = 6;
  const barW = (innerW - barGap * (block.bars.length - 1)) / block.bars.length;

  return (
    <div className="rounded-lg border border-border bg-background/60 p-3">
      <div className="mb-2 text-xs font-medium text-muted-foreground">
        {block.label}
      </div>
      <svg
        viewBox={`0 0 ${W} ${H}`}
        className="h-24 w-full"
        role="img"
        aria-label={`${block.label} bar chart`}
      >
        <line
          x1={PAD_X}
          x2={W - PAD_X}
          y1={H - PAD_Y}
          y2={H - PAD_Y}
          className="stroke-border"
        />
        {block.bars.map((bar, i) => {
          const h = ((bar.value / vRange) * (H - PAD_Y * 2));
          const x = PAD_X + i * (barW + barGap);
          const y = H - PAD_Y - h;
          return (
            <g key={i}>
              <rect
                x={x}
                y={y}
                width={barW}
                height={Math.max(h, 1)}
                rx={2}
                className={TONE_FILL[bar.tone]}
              />
              <text
                x={x + barW / 2}
                y={y - 3}
                textAnchor="middle"
                className="fill-muted-foreground text-[9px]"
              >
                {bar.value}
              </text>
              <text
                x={x + barW / 2}
                y={H - PAD_Y + 12}
                textAnchor="middle"
                className="fill-muted-foreground text-[9px]"
              >
                {bar.label}
              </text>
            </g>
          );
        })}
      </svg>
    </div>
  );
}
