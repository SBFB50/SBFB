// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Zustand store for the shell's list of "known coordinators"
 * and the currently-active one. Persisted to localStorage so
 * the user's coordinator list survives page reloads.
 *
 * Sprint 5 decision D4: the shell does NOT scan ports or read
 * the filesystem. The user either enters a coordinator URL in
 * the onboarding dialog, or an already-known coordinator
 * publishes a sibling via `GET /shell/discover`. Both paths
 * land here and update the persisted list.
 *
 * Schema is versioned via the `name` field below
 * (`nexus-grid:shell:v1`). Any breaking change to
 * KnownCoordinator means bumping the name to `v2` so existing
 * clients migrate cleanly.
 */

import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";
import { normalizeApiUrl } from "@/api/coordinator";

export interface KnownCoordinator {
  /** Normalized URL, e.g. `http://127.0.0.1:8765` (no trailing slash). */
  url: string;
  /**
   * Display name — either the `project_name` discovered via
   * `/shell/discover` or a user-supplied nickname from the
   * "Add coordinator" dialog. Empty string means the shell
   * will fall back to the URL itself.
   */
  nickname: string;
  /**
   * Optional `node_id` from `/health`. Used to render the
   * truncated identifier in the sidebar and to dedupe entries
   * that come in via different URLs (e.g. ipv6 vs ipv4).
   */
  nodeId: string | null;
}

interface ProjectStoreState {
  knownCoordinators: KnownCoordinator[];
  activeCoordinatorUrl: string | null;

  addCoordinator: (
    url: string,
    opts?: { nickname?: string; nodeId?: string | null },
  ) => KnownCoordinator;
  removeCoordinator: (url: string) => void;
  setActive: (url: string | null) => void;
  updateCoordinator: (
    url: string,
    patch: Partial<Omit<KnownCoordinator, "url">>,
  ) => void;
  clear: () => void;
}

export const useProjectStore = create<ProjectStoreState>()(
  persist(
    (set, get) => ({
      knownCoordinators: [],
      activeCoordinatorUrl: null,

      addCoordinator: (raw, opts) => {
        const url = normalizeApiUrl(raw);
        const existing = get().knownCoordinators.find((c) => c.url === url);
        if (existing) {
          // Dedupe by URL — if the caller supplied a nickname or
          // node_id we patch the existing entry in place rather
          // than duplicating it.
          if (opts?.nickname || opts?.nodeId) {
            set((s) => ({
              knownCoordinators: s.knownCoordinators.map((c) =>
                c.url === url
                  ? {
                      ...c,
                      nickname: opts.nickname ?? c.nickname,
                      nodeId: opts.nodeId ?? c.nodeId,
                    }
                  : c,
              ),
            }));
          }
          return existing;
        }
        const entry: KnownCoordinator = {
          url,
          nickname: opts?.nickname ?? "",
          nodeId: opts?.nodeId ?? null,
        };
        set((s) => ({
          knownCoordinators: [...s.knownCoordinators, entry],
          // Auto-select the first coordinator we add so the shell
          // has something to show immediately after onboarding.
          activeCoordinatorUrl: s.activeCoordinatorUrl ?? url,
        }));
        return entry;
      },

      removeCoordinator: (raw) => {
        const url = normalizeApiUrl(raw);
        set((s) => {
          const next = s.knownCoordinators.filter((c) => c.url !== url);
          const active =
            s.activeCoordinatorUrl === url
              ? (next[0]?.url ?? null)
              : s.activeCoordinatorUrl;
          return { knownCoordinators: next, activeCoordinatorUrl: active };
        });
      },

      setActive: (url) => {
        if (url === null) {
          set({ activeCoordinatorUrl: null });
          return;
        }
        const normalized = normalizeApiUrl(url);
        const known = get().knownCoordinators.some((c) => c.url === normalized);
        if (!known) {
          throw new Error(
            `setActive: coordinator ${normalized} is not in knownCoordinators`,
          );
        }
        set({ activeCoordinatorUrl: normalized });
      },

      updateCoordinator: (raw, patch) => {
        const url = normalizeApiUrl(raw);
        set((s) => ({
          knownCoordinators: s.knownCoordinators.map((c) =>
            c.url === url ? { ...c, ...patch } : c,
          ),
        }));
      },

      clear: () => set({ knownCoordinators: [], activeCoordinatorUrl: null }),
    }),
    {
      name: "nexus-grid:shell:v1",
      storage: createJSONStorage(() => localStorage),
    },
  ),
);

/** Helper selector: the active coordinator entry or `null`. */
export function selectActiveCoordinator(
  s: ProjectStoreState,
): KnownCoordinator | null {
  if (!s.activeCoordinatorUrl) return null;
  return (
    s.knownCoordinators.find((c) => c.url === s.activeCoordinatorUrl) ?? null
  );
}
