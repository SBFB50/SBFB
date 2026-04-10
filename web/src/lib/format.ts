/**
 * Pure formatting helpers for the Sprint 5 shell.
 *
 * Every helper is total (never throws), deterministic, and
 * tolerant of `undefined` / `null` inputs so the React render
 * path does not need defensive wrappers. Unit-tested by
 * `format.test.ts` (when vitest lands).
 */

/**
 * Truncate a hex-like identifier (node_id, doc_id, task_id, ...)
 * to the first `chars` characters followed by `…`. Shorter
 * inputs are returned unchanged.
 */
export function formatHash(value: string | null | undefined, chars = 12): string {
  if (!value) return "—";
  if (value.length <= chars) return value;
  return `${value.slice(0, chars)}…`;
}

/** Format a duration in seconds as `1h 23m 45s`. */
export function formatUptime(secs: number): string {
  if (secs <= 0) return "0s";
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = Math.floor(secs % 60);
  const parts: string[] = [];
  if (h > 0) parts.push(`${h}h`);
  if (m > 0 || h > 0) parts.push(`${m}m`);
  parts.push(`${s}s`);
  return parts.join(" ");
}

/**
 * Format an ISO-8601 timestamp relative to now
 * ("il y a 3s", "il y a 2 min"). Accepts ISO strings or unix
 * seconds as number. Returns `"—"` for invalid inputs.
 */
export function formatRelativeTime(
  value: string | number | null | undefined,
): string {
  if (value === null || value === undefined) return "—";
  let ms: number;
  if (typeof value === "number") {
    ms = value * 1000;
  } else {
    const parsed = Date.parse(value);
    if (Number.isNaN(parsed)) return "—";
    ms = parsed;
  }
  const delta = Date.now() - ms;
  const abs = Math.abs(delta);
  const past = delta >= 0;

  const SEC = 1000;
  const MIN = 60 * SEC;
  const HOUR = 60 * MIN;
  const DAY = 24 * HOUR;

  if (abs < 2 * SEC) return past ? "à l'instant" : "dans un instant";
  if (abs < MIN)
    return `${past ? "il y a" : "dans"} ${Math.floor(abs / SEC)} s`;
  if (abs < HOUR)
    return `${past ? "il y a" : "dans"} ${Math.floor(abs / MIN)} min`;
  if (abs < DAY)
    return `${past ? "il y a" : "dans"} ${Math.floor(abs / HOUR)} h`;
  return `${past ? "il y a" : "dans"} ${Math.floor(abs / DAY)} j`;
}

/** Format MiB as `1.2 GiB` or `42 MiB` as appropriate. */
export function formatMemoryMb(mb: number | null | undefined): string {
  if (mb === null || mb === undefined) return "—";
  if (mb >= 1024) return `${(mb / 1024).toFixed(1)} GiB`;
  return `${Math.round(mb)} MiB`;
}
