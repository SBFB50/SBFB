// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 20 Phase B — panic wipe 5-tap keybind.
 *
 * Listens at the window level for the `Ctrl+Shift+Alt+W` chord
 * and triggers the irreversible daemon-side panic wipe once the
 * chord fires **5 times inside 3 seconds**. On any slower
 * cadence or shorter burst the timer window resets silently —
 * there is no UI feedback between taps, by design.
 *
 * ## Why no feedback
 *
 * A visible tap counter ("3/5") turns the panic wipe into a
 * tell-tale gesture for an adversary standing next to the user.
 * The design doc `S20_phase_B_duress_panic_design.md §3` rejects
 * in-UI progress because:
 *
 *   - A guard or customs agent who sees "3/5" understands that
 *     something destructive is being triggered and intervenes.
 *   - Deniability by design: without feedback the gesture looks
 *     like accidental keyboard input.
 *
 * The user learns the gesture at setup via
 * `docs/security/DURESS.md §3.2`, never from the runtime UI.
 *
 * ## Why this chord
 *
 * `Ctrl+Shift+Alt+W` is the most isolated chord we can reach
 * across platforms — W is not bound by Windows Explorer,
 * Chrome, VSCode or tmux. Three modifiers keep the false-trigger
 * rate near zero. See design doc §3 for the trade-off rationale.
 *
 * ## Why not useEffect + setInterval
 *
 * A single `useEffect` hook registers the window listener on
 * mount and tears it down on unmount. The tap-history buffer
 * lives in a `useRef` so React re-renders (state changes
 * elsewhere in the tree) do not reset the gesture progress. The
 * window is a sliding 3-second list of tap timestamps; on each
 * chord fire we prune any entry older than `WINDOW_MS` and count
 * the survivors.
 */

import { useEffect, useRef } from "react";

import { triggerPanicWipe, type DaemonResult, type PanicWipeResponse } from "@/api/daemon";

/** Number of chord presses inside the window that trip the wipe. */
export const REQUIRED_TAPS = 5;

/** Length of the sliding window in milliseconds. */
export const WINDOW_MS = 3000;

export interface PanicWipeKeybindProps {
  /**
   * Base URL of the coordinator `/daemon/*` proxy. The hook
   * forwards it verbatim to `triggerPanicWipe`.
   */
  coordinatorBaseUrl: string;

  /**
   * Optional override injected by unit tests so they can assert
   * that the hook calls the daemon without reaching real
   * network. Defaults to `triggerPanicWipe` from `@/api/daemon`.
   */
  triggerImpl?: (baseUrl: string) => Promise<DaemonResult<PanicWipeResponse>>;
}

/**
 * Register the window-level keydown listener + manage the tap
 * buffer. Renders nothing — the component is invisible and is
 * meant to be mounted once near the `AppShell` root so it
 * survives navigation.
 */
export function PanicWipeKeybind({
  coordinatorBaseUrl,
  triggerImpl,
}: PanicWipeKeybindProps): null {
  const taps = useRef<number[]>([]);
  // Keep refs to the latest props so we can re-read them without
  // re-subscribing the window listener on every render. The
  // ref.current assignment happens in a useEffect (not during
  // render) to avoid the React 19 strict-mode refs-during-render
  // rule — the effect runs right after commit so the listener
  // always reads the freshest value.
  const triggerRef = useRef<typeof triggerImpl>(undefined);
  const baseUrlRef = useRef<string>(coordinatorBaseUrl);

  useEffect(() => {
    triggerRef.current = triggerImpl;
    baseUrlRef.current = coordinatorBaseUrl;
  });

  useEffect(() => {
    function handler(event: KeyboardEvent) {
      if (!isChord(event)) {
        return;
      }
      event.preventDefault();
      event.stopPropagation();

      const now = Date.now();
      // Prune entries outside the window.
      taps.current = taps.current.filter((ts) => now - ts < WINDOW_MS);
      taps.current.push(now);

      if (taps.current.length >= REQUIRED_TAPS) {
        // Reset immediately so a 6th tap cannot double-trigger.
        taps.current = [];
        const runTrigger = triggerRef.current ?? triggerPanicWipe;
        // Fire-and-forget — we do not await. The daemon exits
        // before a response lands in most cases.
        void runTrigger(baseUrlRef.current);
      }
    }

    window.addEventListener("keydown", handler, { capture: true });
    return () => {
      window.removeEventListener("keydown", handler, { capture: true });
    };
  }, []);

  return null;
}

/**
 * Return `true` iff the event represents `Ctrl+Shift+Alt+W`
 * (`Meta` is explicitly rejected so Cmd+Shift+Alt+W on macOS
 * does NOT trip — the gesture is Ctrl-based across every OS for
 * consistency at setup time).
 */
function isChord(event: KeyboardEvent): boolean {
  return (
    event.ctrlKey &&
    event.shiftKey &&
    event.altKey &&
    !event.metaKey &&
    event.key.toLowerCase() === "w"
  );
}
