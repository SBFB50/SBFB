// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Unit tests for the same-origin coordinator auto-registration
 * boot helper. The hotfix removes the "Aucun coordinateur"
 * empty-state when the serving daemon is reachable on the current
 * origin, while staying a no-op once the user manages the list
 * manually or when no same-origin daemon answers.
 *
 * `getDaemonInfo` is mocked so the helper can be exercised without
 * a live daemon; the real Zustand store is used and reset between
 * cases (its singleton would otherwise leak state).
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  autoRegisterLocalCoordinator,
  LOCAL_COORDINATOR_NICKNAME,
} from "../bootstrap";
import { getDaemonInfo, type DaemonInfo } from "@/api/daemon";
import { useProjectStore } from "@/stores/projectStore";

vi.mock("@/api/daemon", () => ({
  getDaemonInfo: vi.fn(),
}));

const mockGetDaemonInfo = vi.mocked(getDaemonInfo);

function daemonInfo(nodeId: string): DaemonInfo {
  return {
    schema_version: 1,
    node_id: nodeId,
    daemon_version: "1.0.0",
    uptime_secs: 1,
    started_at: "2026-06-07T00:00:00Z",
    last_updated_at: "2026-06-07T00:00:00Z",
    api_host: "127.0.0.1",
    api_port: 8787,
    subscribed_curators: [],
    known_lists: 0,
    known_browse_entries: 0,
  };
}

beforeEach(() => {
  useProjectStore.getState().clear();
  localStorage.clear();
  mockGetDaemonInfo.mockReset();
});

afterEach(() => {
  useProjectStore.getState().clear();
  localStorage.clear();
});

describe("autoRegisterLocalCoordinator", () => {
  it("seeds the same-origin daemon when the list is empty and the daemon is reachable", async () => {
    mockGetDaemonInfo.mockResolvedValue({
      kind: "data",
      status: 200,
      body: daemonInfo("node-xyz"),
    });

    await autoRegisterLocalCoordinator();

    const { knownCoordinators, activeCoordinatorUrl } =
      useProjectStore.getState();
    expect(knownCoordinators).toHaveLength(1);
    expect(knownCoordinators[0]).toEqual({
      url: window.location.origin,
      nickname: LOCAL_COORDINATOR_NICKNAME,
      nodeId: "node-xyz",
    });
    // auto-selected so Browse renders immediately instead of the wall
    expect(activeCoordinatorUrl).toBe(window.location.origin);
    // probed exactly the current origin — no port scan (Sprint 5 D4)
    expect(mockGetDaemonInfo).toHaveBeenCalledTimes(1);
    expect(mockGetDaemonInfo).toHaveBeenCalledWith(window.location.origin);
  });

  it("is a no-op (and does not probe) when the user already has a coordinator", async () => {
    useProjectStore.getState().addCoordinator("http://127.0.0.1:9999", {
      nickname: "manual",
      nodeId: "manual-node",
    });

    await autoRegisterLocalCoordinator();

    const list = useProjectStore.getState().knownCoordinators;
    expect(list).toHaveLength(1);
    expect(list[0].url).toBe("http://127.0.0.1:9999");
    expect(mockGetDaemonInfo).not.toHaveBeenCalled();
  });

  it("does not seed when the same-origin daemon is unavailable", async () => {
    mockGetDaemonInfo.mockResolvedValue({
      kind: "unavailable",
      reason: "daemon unavailable",
    });

    await autoRegisterLocalCoordinator();

    expect(useProjectStore.getState().knownCoordinators).toHaveLength(0);
    expect(useProjectStore.getState().activeCoordinatorUrl).toBeNull();
  });

  it("does not seed when the daemon probe returns an error", async () => {
    mockGetDaemonInfo.mockResolvedValue({ kind: "error", reason: "HTTP 500" });

    await autoRegisterLocalCoordinator();

    expect(useProjectStore.getState().knownCoordinators).toHaveLength(0);
  });

  it("is idempotent — a second call neither probes again nor duplicates", async () => {
    mockGetDaemonInfo.mockResolvedValue({
      kind: "data",
      status: 200,
      body: daemonInfo("node-xyz"),
    });

    await autoRegisterLocalCoordinator();
    await autoRegisterLocalCoordinator();

    expect(useProjectStore.getState().knownCoordinators).toHaveLength(1);
    expect(mockGetDaemonInfo).toHaveBeenCalledTimes(1);
  });
});
