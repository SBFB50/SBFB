/**
 * Global Ctrl+K / Cmd+K listener for the command palette.
 *
 * Mounted once at the AppShell root so the shortcut works on
 * every page regardless of focus. Escape closes the palette via
 * the underlying Radix/base-ui Dialog.
 */

import { useCallback, useEffect, useState } from "react";

export function useCommandPalette() {
  const [open, setOpen] = useState(false);

  const toggle = useCallback(() => setOpen((prev) => !prev), []);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "k" && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        setOpen((prev) => !prev);
      }
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, []);

  return { open, setOpen, toggle };
}
