import type { TabBlockChartLine } from "../schema";

const W = 400;
const H = 120;
const PAD_X = 32;
const PAD_Y = 16;

export function ChartLineBlock({ block }: { block: TabBlockChartLine }) {
  if (block.points.length === 0) {
    return (
      <div className="rounded-lg border border-border bg-background/60 p-3 text-xs italic text-muted-foreground">
        {block.label} — (aucun point)
      </div>
    );
  }
  const ys = block.points.map((p) => p.y);
  const yMin = Math.min(...ys);
  const yMax = Math.max(...ys);
  const yRange = yMax - yMin || 1;
  const stepX = (W - PAD_X * 2) / Math.max(block.points.length - 1, 1);
  const toX = (i: number) => PAD_X + i * stepX;
  const toY = (y: number) =>
    H - PAD_Y - ((y - yMin) / yRange) * (H - PAD_Y * 2);
  const path = block.points
    .map((p, i) => `${i === 0 ? "M" : "L"}${toX(i).toFixed(1)},${toY(p.y).toFixed(1)}`)
    .join(" ");
  const ticks = [yMax, (yMax + yMin) / 2, yMin];

  return (
    <div className="rounded-lg border border-border bg-background/60 p-3">
      <div className="mb-2 flex items-baseline justify-between">
        <span className="text-xs font-medium text-muted-foreground">
          {block.label}
        </span>
        {block.y_unit && (
          <span className="text-[10px] text-muted-foreground">
            {block.y_unit}
          </span>
        )}
      </div>
      <svg
        viewBox={`0 0 ${W} ${H}`}
        className="h-24 w-full"
        role="img"
        aria-label={`${block.label} line chart`}
      >
        {ticks.map((t, i) => (
          <g key={i}>
            <line
              x1={PAD_X}
              x2={W - PAD_X / 2}
              y1={toY(t)}
              y2={toY(t)}
              className="stroke-border"
              strokeDasharray="2 3"
            />
            <text
              x={PAD_X - 4}
              y={toY(t) + 3}
              textAnchor="end"
              className="fill-muted-foreground text-[9px]"
            >
              {t.toFixed(1)}
            </text>
          </g>
        ))}
        <path
          d={path}
          fill="none"
          strokeWidth={2}
          className="stroke-primary"
        />
        {block.points.map((p, i) => (
          <circle
            key={i}
            cx={toX(i)}
            cy={toY(p.y)}
            r={2.5}
            className="fill-primary"
          />
        ))}
        {block.points.map((p, i) =>
          i === 0 || i === block.points.length - 1 ? (
            <text
              key={`lbl-${i}`}
              x={toX(i)}
              y={H - 2}
              textAnchor={i === 0 ? "start" : "end"}
              className="fill-muted-foreground text-[9px]"
            >
              {p.x}
            </text>
          ) : null,
        )}
      </svg>
    </div>
  );
}
