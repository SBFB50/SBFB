/**
 * Sprint 8 Phase E helper — narrow an `invokeAppCommand` result
 * to a `{navigation: {path}}` shape without pulling Zod into
 * the palette hot path.
 *
 * Lives in its own module (rather than inside
 * `CommandPalette.tsx`) so React Fast Refresh's "only export
 * components" rule stays clean on the palette file.
 */

export function extractNavigationPath(result: unknown): string | null {
  if (typeof result !== "object" || result === null) return null;
  const nav = (result as { navigation?: unknown }).navigation;
  if (typeof nav !== "object" || nav === null) return null;
  const path = (nav as { path?: unknown }).path;
  if (typeof path !== "string" || path.length === 0) return null;
  return path;
}
