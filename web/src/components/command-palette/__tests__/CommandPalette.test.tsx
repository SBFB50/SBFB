/**
 * Sprint 8 Phase E — CommandPalette component tests.
 *
 * Focus: the new "App: <name>" group that fetches
 * `@nexus_command` descriptors from the active coordinator and
 * forwards invocations through `invokeAppCommand`. We stub the
 * API layer at the module boundary (vi.mock) so the test does
 * not spin up a real coordinator and stays entirely
 * deterministic.
 *
 * Cases covered:
 *  - no active coordinator → palette renders only Navigation +
 *    Actions groups (no crash from the optional `appsQuery`)
 *  - 1 active coordinator + 0 commands-enabled apps → no App
 *    group emitted (graceful empty)
 *  - 1 active coordinator + 1 gov app with 4 commands →
 *    commands appear under "App : gov"
 *  - select command → invokeAppCommand called, navigate fired
 *    when the handler returned `{navigation: {path}}`
 *  - select command → handler returns no navigation → palette
 *    closes, navigate NOT called (no stray routing)
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import { CommandPalette } from "../CommandPalette";
import { useProjectStore } from "@/stores/projectStore";

// ---------------------------------------------------------------
// Mock the coordinator API at the module boundary. `vi.mock` is
// hoisted so these replacements are in place before the SUT
// imports `@/api/coordinator`.
// ---------------------------------------------------------------

// Keep the real module so collaborators (`normalizeCoordinatorUrl`
// pulled in by the Zustand store) still work; only stub the three
// Sprint 8 Phase A/E endpoints that the palette's React Query
// calls target.
vi.mock("@/api/coordinator", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/api/coordinator")>();
  return {
    ...actual,
    listApps: vi.fn(),
    listAppCommands: vi.fn(),
    invokeAppCommand: vi.fn(),
  };
});

import * as api from "@/api/coordinator";

const mockedListApps = vi.mocked(api.listApps);
const mockedListAppCommands = vi.mocked(api.listAppCommands);
const mockedInvokeAppCommand = vi.mocked(api.invokeAppCommand);

// Mock react-router's `useNavigate` so the test can assert what
// path the palette forwards after an invoke. We keep the other
// react-router exports (MemoryRouter) real so `<CommandPalette>`
// renders inside a live routing context.
const navigateSpy = vi.fn();
vi.mock("react-router-dom", async () => {
  const actual =
    await vi.importActual<typeof import("react-router-dom")>(
      "react-router-dom",
    );
  return {
    ...actual,
    useNavigate: () => navigateSpy,
  };
});

// ---------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------

function makePalette(open = true) {
  return { open, setOpen: vi.fn(), toggle: vi.fn() };
}

function renderPalette(paletteOpen = true) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
      },
    },
  });
  const onAddCoordinator = vi.fn();
  const palette = makePalette(paletteOpen);
  render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={["/my-projects"]}>
        <CommandPalette
          palette={palette}
          onAddCoordinator={onAddCoordinator}
        />
      </MemoryRouter>
    </QueryClientProvider>,
  );
  return { palette, onAddCoordinator };
}

function seedActiveCoordinator() {
  act(() => {
    useProjectStore.getState().clear();
    useProjectStore
      .getState()
      .addCoordinator("http://127.0.0.1:8765", { nickname: "alpha" });
  });
}

// ---------------------------------------------------------------
// Cases
// ---------------------------------------------------------------

describe("CommandPalette — App group (Sprint 8 Phase E)", () => {
  beforeEach(() => {
    act(() => {
      useProjectStore.getState().clear();
    });
    localStorage.clear();
    navigateSpy.mockReset();
    mockedListApps.mockReset();
    mockedListAppCommands.mockReset();
    mockedInvokeAppCommand.mockReset();
  });

  afterEach(() => {
    act(() => {
      useProjectStore.getState().clear();
    });
    localStorage.clear();
  });

  it("renders Navigation + Actions even with no active coordinator", async () => {
    renderPalette();

    expect(await screen.findByText("Navigation")).toBeInTheDocument();
    expect(screen.getByText("Actions")).toBeInTheDocument();
    expect(screen.queryByText(/^App :/)).not.toBeInTheDocument();
    // No API call fired when there's no active coord.
    expect(mockedListApps).not.toHaveBeenCalled();
  });

  it("does not render an App group when no app has commands", async () => {
    seedActiveCoordinator();
    mockedListApps.mockResolvedValue({
      count: 1,
      apps: [
        {
          name: "gov",
          version: "0.3.0",
          description: "Gov app",
          routes: 1,
          workers: 3,
          tabs: 19,
          commands: 0,
        },
      ],
    });

    renderPalette();

    // listApps fires, but since commands === 0 no AppCommandsGroup
    // is mounted → no listAppCommands call, no "App :" heading.
    await waitFor(() => expect(mockedListApps).toHaveBeenCalledTimes(1));
    expect(mockedListAppCommands).not.toHaveBeenCalled();
    expect(screen.queryByText(/^App :/)).not.toBeInTheDocument();
  });

  it("renders the gov App group with four commands", async () => {
    seedActiveCoordinator();
    mockedListApps.mockResolvedValue({
      count: 1,
      apps: [
        {
          name: "gov",
          version: "0.3.0",
          description: "Gov app",
          routes: 1,
          workers: 3,
          tabs: 19,
          commands: 4,
        },
      ],
    });
    mockedListAppCommands.mockResolvedValue([
      {
        schema_version: 1,
        name: "detect_contradictions",
        description: "Détecter les contradictions politiques",
        icon: "alert-octagon",
        group: "Gov",
      },
      {
        schema_version: 1,
        name: "new_scan",
        description: "Lancer un nouveau scan des politiciens",
        icon: "radar",
        group: "Gov",
      },
      {
        schema_version: 1,
        name: "search_factchecks",
        description: "Rechercher dans les fact-checks",
        icon: "check-circle",
        group: "Gov",
      },
      {
        schema_version: 1,
        name: "view_alerts",
        description: "Consulter les alertes récentes",
        icon: "bell",
        group: "Gov",
      },
    ]);

    renderPalette();

    await waitFor(() =>
      expect(screen.getByText(/^App : gov$/)).toBeInTheDocument(),
    );
    expect(
      screen.getByText("Détecter les contradictions politiques"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Lancer un nouveau scan des politiciens"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Rechercher dans les fact-checks"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Consulter les alertes récentes"),
    ).toBeInTheDocument();

    expect(mockedListAppCommands).toHaveBeenCalledWith(
      "http://127.0.0.1:8765",
      "gov",
    );
  });

  it("invokes the command and navigates when the handler returns navigation", async () => {
    seedActiveCoordinator();
    mockedListApps.mockResolvedValue({
      count: 1,
      apps: [
        {
          name: "gov",
          version: "0.3.0",
          description: "Gov app",
          routes: 1,
          workers: 3,
          tabs: 19,
          commands: 1,
        },
      ],
    });
    mockedListAppCommands.mockResolvedValue([
      {
        schema_version: 1,
        name: "detect_contradictions",
        description: "Détecter les contradictions politiques",
        icon: "alert-octagon",
        group: "Gov",
      },
    ]);
    mockedInvokeAppCommand.mockResolvedValue({
      result: { navigation: { path: "/app/gov/tabs/Contradictions" } },
    });

    const { palette } = renderPalette();

    await waitFor(() =>
      expect(
        screen.getByText("Détecter les contradictions politiques"),
      ).toBeInTheDocument(),
    );
    const user = userEvent.setup();
    await user.click(
      screen.getByText("Détecter les contradictions politiques"),
    );

    await waitFor(() => {
      expect(mockedInvokeAppCommand).toHaveBeenCalledWith(
        "http://127.0.0.1:8765",
        "gov",
        "detect_contradictions",
      );
    });
    expect(navigateSpy).toHaveBeenCalledWith("/app/gov/tabs/Contradictions");
    expect(palette.setOpen).toHaveBeenCalledWith(false);
  });

  it("shows inline error state and keeps palette open on invoke failure (T11)", async () => {
    seedActiveCoordinator();
    mockedListApps.mockResolvedValue({
      count: 1,
      apps: [
        {
          name: "gov",
          version: "0.3.0",
          description: "Gov app",
          routes: 1,
          workers: 3,
          tabs: 19,
          commands: 1,
        },
      ],
    });
    mockedListAppCommands.mockResolvedValue([
      {
        schema_version: 1,
        name: "detect_contradictions",
        description: "Détecter les contradictions politiques",
        icon: "alert-octagon",
        group: "Gov",
      },
    ]);
    mockedInvokeAppCommand.mockRejectedValue(
      new Error("coordinator 500: boom"),
    );

    const { palette } = renderPalette();

    await waitFor(() =>
      expect(
        screen.getByText("Détecter les contradictions politiques"),
      ).toBeInTheDocument(),
    );
    const user = userEvent.setup();
    await user.click(
      screen.getByText("Détecter les contradictions politiques"),
    );

    // Error branch: inline message renders next to the row,
    // palette does NOT close (setOpen(false) is never called
    // on error), and navigate is never fired.
    await waitFor(() =>
      expect(
        screen.getByTestId("palette-cmd-error-gov-detect_contradictions"),
      ).toBeInTheDocument(),
    );
    expect(
      screen.getByTestId("palette-cmd-error-gov-detect_contradictions"),
    ).toHaveTextContent("coordinator 500: boom");
    expect(palette.setOpen).not.toHaveBeenCalledWith(false);
    expect(navigateSpy).not.toHaveBeenCalled();
  });

  it("allows retrying an errored command (T11)", async () => {
    seedActiveCoordinator();
    mockedListApps.mockResolvedValue({
      count: 1,
      apps: [
        {
          name: "gov",
          version: "0.3.0",
          description: "Gov app",
          routes: 1,
          workers: 3,
          tabs: 19,
          commands: 1,
        },
      ],
    });
    mockedListAppCommands.mockResolvedValue([
      {
        schema_version: 1,
        name: "detect_contradictions",
        description: "Détecter les contradictions politiques",
        icon: "alert-octagon",
        group: "Gov",
      },
    ]);
    // First call throws, second call succeeds.
    mockedInvokeAppCommand
      .mockRejectedValueOnce(new Error("temporary failure"))
      .mockResolvedValueOnce({
        result: { navigation: { path: "/app/gov/tabs/Contradictions" } },
      });

    const { palette } = renderPalette();

    await waitFor(() =>
      expect(
        screen.getByText("Détecter les contradictions politiques"),
      ).toBeInTheDocument(),
    );
    const user = userEvent.setup();

    // First click → errors out, palette stays open with inline banner.
    await user.click(
      screen.getByText("Détecter les contradictions politiques"),
    );
    await waitFor(() =>
      expect(
        screen.getByTestId("palette-cmd-error-gov-detect_contradictions"),
      ).toBeInTheDocument(),
    );
    expect(palette.setOpen).not.toHaveBeenCalledWith(false);

    // Second click → mock resolves, palette closes, navigate fires.
    await user.click(
      screen.getByText("Détecter les contradictions politiques"),
    );
    await waitFor(() =>
      expect(mockedInvokeAppCommand).toHaveBeenCalledTimes(2),
    );
    expect(navigateSpy).toHaveBeenCalledWith("/app/gov/tabs/Contradictions");
    expect(palette.setOpen).toHaveBeenCalledWith(false);
  });

  it("closes the palette without navigating when the handler returns no navigation", async () => {
    seedActiveCoordinator();
    mockedListApps.mockResolvedValue({
      count: 1,
      apps: [
        {
          name: "gov",
          version: "0.3.0",
          description: "Gov app",
          routes: 1,
          workers: 3,
          tabs: 19,
          commands: 1,
        },
      ],
    });
    mockedListAppCommands.mockResolvedValue([
      {
        schema_version: 1,
        name: "noop",
        description: "Command sans navigation",
        icon: "sparkles",
        group: "Gov",
      },
    ]);
    mockedInvokeAppCommand.mockResolvedValue({ result: null });

    const { palette } = renderPalette();

    await waitFor(() =>
      expect(screen.getByText("Command sans navigation")).toBeInTheDocument(),
    );
    const user = userEvent.setup();
    await user.click(screen.getByText("Command sans navigation"));

    await waitFor(() => {
      expect(mockedInvokeAppCommand).toHaveBeenCalled();
    });
    expect(navigateSpy).not.toHaveBeenCalled();
    expect(palette.setOpen).toHaveBeenCalledWith(false);
  });
});

describe("extractNavigationPath helper (Sprint 8 Phase E)", () => {
  it("returns the path for a well-formed payload", async () => {
    const { extractNavigationPath } = await import(
      "../extractNavigationPath"
    );
    expect(
      extractNavigationPath({ navigation: { path: "/app/gov/tabs/Scan" } }),
    ).toBe("/app/gov/tabs/Scan");
  });

  it("returns null for non-objects, missing navigation, or empty path", async () => {
    const { extractNavigationPath } = await import(
      "../extractNavigationPath"
    );
    expect(extractNavigationPath(null)).toBeNull();
    expect(extractNavigationPath("hello")).toBeNull();
    expect(extractNavigationPath({})).toBeNull();
    expect(extractNavigationPath({ navigation: null })).toBeNull();
    expect(extractNavigationPath({ navigation: {} })).toBeNull();
    expect(extractNavigationPath({ navigation: { path: "" } })).toBeNull();
    expect(extractNavigationPath({ navigation: { path: 42 } })).toBeNull();
  });
});
