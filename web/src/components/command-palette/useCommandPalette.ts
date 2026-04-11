/**
 * Global Ctrl+K / Cmd+K listener for the command palette.
 *
 * Mounted once at the AppShell root so the shortcut works on
 * every page regardless of focus. Escape closes the palette via
 * the underlying Radix/base-ui Dialog.
 *
 * Key matching: uses `e.code === "KeyK"` because `e.key` is
 * case-sensitive ("k" vs "K" under caps lock) AND
 * layout-dependent (Dvorak, Czech, Russian return a different
 * character for the same physical key). `e.code` is the physical
 * key identifier and stays stable across both.
 */

import { useCallback, useEffect, useState } from "react";

export function useCommandPalette() {
  const [open, setOpen] = useState(false);

  const toggle = useCallback(() => setOpen((prev) => !prev), []);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.code === "KeyK" && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        setOpen((prev) => !prev);
      }
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, []);

  return { open, setOpen, toggle };
}
