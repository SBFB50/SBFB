// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Shell boot helper: auto-register the daemon that serves this
 * shell (same origin) as the default coordinator.
 *
 * Why: the daemon binds an ephemeral loopback port and serves the
 * React shell + its bearer token (`GET /auth/token`) on that same
 * origin (`nexus-shell-daemon start --web-root`). Yet the
 * coordinator list lives in per-origin localStorage, so a fresh
 * browser profile — or simply a new port after a daemon restart —
 * starts empty and Browse shows the "Aucun coordinateur" wall even
 * though the serving daemon is right there. This seeds the list
 * with the current origin so the common single-node case lands
 * straight on Browse.
 *
 * Respects Sprint 5 decision D4 ("the shell does NOT scan ports or
 * read the filesystem"): it probes a single authenticated daemon
 * API (`GET /api/daemon/info`) on the SAME origin that already
 * served the shell and handed out the token — no port scan, no FS
 * access. It seeds only when the user has no coordinators yet;
 * manual add/remove via AddCoordinatorDialog stays the source of
 * truth afterwards.
 */

import { getDaemonInfo } from "@/api/daemon";
import { useProjectStore } from "@/stores/projectStore";

/** Display name for the auto-seeded same-origin coordinator. */
export const LOCAL_COORDINATOR_NICKNAME = "Ce nœud (local)";

/**
 * Seed the same-origin daemon as a coordinator when the shell has
 * none yet. No-op when (a) the user already has coordinators,
 * (b) there is no DOM `window`, (c) the origin is not http(s)
 * (e.g. `file://`), or (d) the same-origin daemon is unreachable
 * (e.g. the shell is served by a launcher whose daemon lives on a
 * different origin) — the manual "Ajouter un coordinateur" flow
 * stays available in that case.
 *
 * Safe to call more than once: the empty-list guard short-circuits
 * after the first seed and `addCoordinator` dedupes by URL.
 *
 * Call this AFTER the bearer token resolves so the probe carries
 * it and does not 401 transiently.
 */
export async function autoRegisterLocalCoordinator(): Promise<void> {
  const store = useProjectStore.getState();
  if (store.knownCoordinators.length > 0) return;

  if (typeof window === "undefined") return;
  const origin = window.location.origin;
  if (!/^https?:\/\//i.test(origin)) return;

  // `getDaemonInfo` returns a `DaemonResult` union for the offline
  // / error cases, but `callDaemon` THROWS `ApiProtocolError` when a
  // 200 body fails the strict `DaemonInfoSchema`. Swallow both so
  // this helper is truly a no-op on any failure (the docstring
  // promise), independent of the caller's own `.catch()`.
  const info = await getDaemonInfo(origin).catch(() => null);
  if (!info || info.kind !== "data") return;

  store.addCoordinator(origin, {
    nickname: LOCAL_COORDINATOR_NICKNAME,
    nodeId: info.body.node_id,
  });
}
