/**
 * Sprint 6 Phase D — Vitest unit tests for the Zustand
 * projectStore. Covers add/remove/setActive/updateCoordinator/
 * clear + the persist middleware + the selectActiveCoordinator
 * selector.
 *
 * Each test runs in isolation by calling `clear()` in
 * `beforeEach` so Zustand's module-level singleton doesn't leak
 * state between cases.
 */

import { afterEach, beforeEach, describe, expect, it } from "vitest";

import {
  selectActiveCoordinator,
  useProjectStore,
} from "../projectStore";

beforeEach(() => {
  useProjectStore.getState().clear();
  localStorage.clear();
});

afterEach(() => {
  useProjectStore.getState().clear();
  localStorage.clear();
});

describe("addCoordinator", () => {
  it("adds a coordinator with nickname + nodeId", () => {
    const entry = useProjectStore
      .getState()
      .addCoordinator("http://127.0.0.1:8765", {
        nickname: "local",
        nodeId: "abc123",
      });
    expect(entry).toEqual({
      url: "http://127.0.0.1:8765",
      nickname: "local",
      nodeId: "abc123",
    });
    expect(useProjectStore.getState().knownCoordinators).toHaveLength(1);
  });

  it("auto-selects the first added coordinator", () => {
    useProjectStore.getState().addCoordinator("http://127.0.0.1:8765");
    expect(useProjectStore.getState().activeCoordinatorUrl).toBe(
      "http://127.0.0.1:8765",
    );
  });

  it("normalises trailing slashes", () => {
    const entry = useProjectStore
      .getState()
      .addCoordinator("http://127.0.0.1:8765/");
    expect(entry.url).toBe("http://127.0.0.1:8765");
  });

  it("dedupes by normalised URL", () => {
    useProjectStore.getState().addCoordinator("http://127.0.0.1:8765");
    useProjectStore.getState().addCoordinator("http://127.0.0.1:8765/");
    expect(useProjectStore.getState().knownCoordinators).toHaveLength(1);
  });

  it("patches nickname / nodeId when re-adding an existing URL", () => {
    useProjectStore.getState().addCoordinator("http://127.0.0.1:8765");
    useProjectStore
      .getState()
      .addCoordinator("http://127.0.0.1:8765", {
        nickname: "patched",
        nodeId: "new-id",
      });
    const list = useProjectStore.getState().knownCoordinators;
    expect(list).toHaveLength(1);
    expect(list[0].nickname).toBe("patched");
    expect(list[0].nodeId).toBe("new-id");
  });

  it("throws on empty URL via normalizer", () => {
    expect(() => useProjectStore.getState().addCoordinator("   ")).toThrow();
  });
});

describe("removeCoordinator", () => {
  it("removes the entry", () => {
    useProjectStore.getState().addCoordinator("http://127.0.0.1:8765");
    useProjectStore
      .getState()
      .removeCoordinator("http://127.0.0.1:8765");
    expect(useProjectStore.getState().knownCoordinators).toHaveLength(0);
  });

  it("re-assigns active to the next coordinator when active is removed", () => {
    useProjectStore.getState().addCoordinator("http://127.0.0.1:8765");
    useProjectStore.getState().addCoordinator("http://127.0.0.1:8766");
    useProjectStore.getState().removeCoordinator("http://127.0.0.1:8765");
    expect(useProjectStore.getState().activeCoordinatorUrl).toBe(
      "http://127.0.0.1:8766",
    );
  });

  it("sets active to null when the last coordinator is removed", () => {
    useProjectStore.getState().addCoordinator("http://127.0.0.1:8765");
    useProjectStore.getState().removeCoordinator("http://127.0.0.1:8765");
    expect(useProjectStore.getState().activeCoordinatorUrl).toBeNull();
  });

  it("leaves the active URL untouched when removing a different entry", () => {
    useProjectStore.getState().addCoordinator("http://127.0.0.1:8765");
    useProjectStore.getState().addCoordinator("http://127.0.0.1:8766");
    useProjectStore.getState().removeCoordinator("http://127.0.0.1:8766");
    expect(useProjectStore.getState().activeCoordinatorUrl).toBe(
      "http://127.0.0.1:8765",
    );
  });
});

describe("setActive", () => {
  it("clears the active URL when passed null", () => {
    useProjectStore.getState().addCoordinator("http://127.0.0.1:8765");
    useProjectStore.getState().setActive(null);
    expect(useProjectStore.getState().activeCoordinatorUrl).toBeNull();
  });

  it("sets the active URL when the coordinator is known", () => {
    useProjectStore.getState().addCoordinator("http://127.0.0.1:8765");
    useProjectStore.getState().addCoordinator("http://127.0.0.1:8766");
    useProjectStore.getState().setActive("http://127.0.0.1:8766");
    expect(useProjectStore.getState().activeCoordinatorUrl).toBe(
      "http://127.0.0.1:8766",
    );
  });

  it("throws when the URL is unknown", () => {
    expect(() =>
      useProjectStore.getState().setActive("http://127.0.0.1:9999"),
    ).toThrow(/not in knownCoordinators/);
  });

  it("normalises trailing slash before lookup", () => {
    useProjectStore.getState().addCoordinator("http://127.0.0.1:8765");
    useProjectStore.getState().setActive("http://127.0.0.1:8765/");
    expect(useProjectStore.getState().activeCoordinatorUrl).toBe(
      "http://127.0.0.1:8765",
    );
  });
});

describe("updateCoordinator", () => {
  it("patches nickname and nodeId", () => {
    useProjectStore.getState().addCoordinator("http://127.0.0.1:8765");
    useProjectStore.getState().updateCoordinator("http://127.0.0.1:8765", {
      nickname: "updated",
      nodeId: "node-xyz",
    });
    const list = useProjectStore.getState().knownCoordinators;
    expect(list[0].nickname).toBe("updated");
    expect(list[0].nodeId).toBe("node-xyz");
  });

  it("is a no-op when the URL is unknown", () => {
    useProjectStore.getState().addCoordinator("http://127.0.0.1:8765");
    useProjectStore.getState().updateCoordinator("http://127.0.0.1:9999", {
      nickname: "none",
    });
    expect(useProjectStore.getState().knownCoordinators[0].nickname).toBe("");
  });
});

describe("selectActiveCoordinator", () => {
  it("returns null when activeCoordinatorUrl is null", () => {
    expect(selectActiveCoordinator(useProjectStore.getState())).toBeNull();
  });

  it("returns the active entry when set", () => {
    useProjectStore.getState().addCoordinator("http://127.0.0.1:8765", {
      nickname: "live",
    });
    const active = selectActiveCoordinator(useProjectStore.getState());
    expect(active).not.toBeNull();
    expect(active?.nickname).toBe("live");
  });

  it("returns null when the active URL is stale (defensive)", () => {
    useProjectStore.getState().addCoordinator("http://127.0.0.1:8765");
    // Forcefully desync the active URL from the list
    useProjectStore.setState({
      activeCoordinatorUrl: "http://127.0.0.1:8766",
    });
    expect(selectActiveCoordinator(useProjectStore.getState())).toBeNull();
  });
});

describe("persist middleware", () => {
  it("writes to localStorage under the versioned key", () => {
    useProjectStore.getState().addCoordinator("http://127.0.0.1:8765", {
      nickname: "saved",
    });
    const raw = localStorage.getItem("nexus-grid:shell:v1");
    expect(raw).not.toBeNull();
    const parsed = JSON.parse(raw!);
    expect(parsed.state.knownCoordinators).toHaveLength(1);
    expect(parsed.state.knownCoordinators[0].nickname).toBe("saved");
  });

  it("clear() empties the store and the underlying state", () => {
    useProjectStore.getState().addCoordinator("http://127.0.0.1:8765");
    useProjectStore.getState().clear();
    expect(useProjectStore.getState().knownCoordinators).toHaveLength(0);
    expect(useProjectStore.getState().activeCoordinatorUrl).toBeNull();
  });
});
