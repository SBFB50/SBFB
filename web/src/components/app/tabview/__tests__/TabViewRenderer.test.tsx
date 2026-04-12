// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 6 Phase D — TabView renderer tests.
 *
 * One test per block kind (11 total) plus the schema validation
 * edge cases. Tests use @testing-library/react + jsdom. React
 * Router's <MemoryRouter> wraps renders that exercise the
 * <ButtonBlock> path (which calls useNavigate).
 *
 * Sprint 8 Phase A: adds coverage for the new `task_submit`
 * wiring in ButtonBlock (D1 + D4) — asserts the context-less
 * error path and the context-ful happy path with a stubbed
 * submitAppTask.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";

import { TabViewRenderer } from "../TabViewRenderer";
import { TabAppContext } from "../TabAppContext";
import { TabViewSchema, parseTabView, type TabView } from "../schema";

function wrap(tabView: TabView) {
  return render(
    <MemoryRouter>
      <TabViewRenderer tabView={tabView} />
    </MemoryRouter>,
  );
}

function make(blocks: TabView["blocks"], title?: string): TabView {
  return {
    schema_version: 1,
    tab_name: "test",
    title,
    blocks,
  };
}

describe("TabViewRenderer — top level", () => {
  it("renders the title when provided", () => {
    wrap(make([], "My title"));
    expect(screen.getByText("My title")).toBeInTheDocument();
  });

  it("renders the empty-state line when no blocks", () => {
    wrap(make([]));
    expect(screen.getByText(/aucun bloc à afficher/)).toBeInTheDocument();
  });
});

describe("TabViewRenderer — each block kind", () => {
  it("renders a heading block", () => {
    wrap(make([{ kind: "heading", level: 1, text: "Hello H1" }]));
    expect(screen.getByText("Hello H1")).toBeInTheDocument();
  });

  it("renders a text block with muted variant", () => {
    wrap(make([{ kind: "text", text: "body text", muted: true }]));
    expect(screen.getByText("body text")).toBeInTheDocument();
  });

  it("renders a kv block with hint", () => {
    wrap(
      make([
        {
          kind: "kv",
          items: [
            { label: "Version", value: "1.0", hint: "last release" },
            { label: "Count", value: 42 },
          ],
        },
      ]),
    );
    expect(screen.getByText("Version")).toBeInTheDocument();
    expect(screen.getByText("1.0")).toBeInTheDocument();
    expect(screen.getByText("last release")).toBeInTheDocument();
    expect(screen.getByText("Count")).toBeInTheDocument();
    expect(screen.getByText("42")).toBeInTheDocument();
  });

  it("renders a metric block with delta and unit", () => {
    wrap(
      make([
        {
          kind: "metric",
          label: "Tasks",
          value: 42,
          delta: 5,
          unit: "total",
          tone: "ok",
        },
      ]),
    );
    expect(screen.getByText("Tasks")).toBeInTheDocument();
    expect(screen.getByText("42")).toBeInTheDocument();
    expect(screen.getByText("total")).toBeInTheDocument();
    expect(screen.getByText("+5")).toBeInTheDocument();
  });

  it("renders a metric block with negative delta", () => {
    wrap(
      make([
        {
          kind: "metric",
          label: "Errors",
          value: 3,
          delta: -2,
          tone: "neutral",
        },
      ]),
    );
    expect(screen.getByText("-2")).toBeInTheDocument();
  });

  it("renders a table block with rows", () => {
    wrap(
      make([
        {
          kind: "table",
          columns: [
            { key: "name", label: "Name", align: "left" },
            { key: "score", label: "Score", align: "right" },
          ],
          rows: [
            { name: "alice", score: 10 },
            { name: "bob", score: null },
          ],
        },
      ]),
    );
    expect(screen.getByText("Name")).toBeInTheDocument();
    expect(screen.getByText("Score")).toBeInTheDocument();
    expect(screen.getByText("alice")).toBeInTheDocument();
    expect(screen.getByText("10")).toBeInTheDocument();
    expect(screen.getByText("bob")).toBeInTheDocument();
    // Null cell renders em-dash
    expect(screen.getAllByText("—").length).toBeGreaterThanOrEqual(1);
  });

  it("renders the table empty-text when rows is empty", () => {
    wrap(
      make([
        {
          kind: "table",
          columns: [{ key: "k", label: "K", align: "left" }],
          rows: [],
          empty_text: "aucune donnée",
        },
      ]),
    );
    expect(screen.getByText("aucune donnée")).toBeInTheDocument();
  });

  it("Phase E polish — cycles column sort asc → desc → none on click", () => {
    wrap(
      make([
        {
          kind: "table",
          columns: [
            { key: "name", label: "Name", align: "left" },
            { key: "score", label: "Score", align: "right" },
          ],
          rows: [
            { name: "charlie", score: 30 },
            { name: "alice", score: 10 },
            { name: "bob", score: null },
          ],
        },
      ]),
    );

    // Server order: charlie, alice, bob.
    const bodyOrder = () =>
      screen
        .getAllByRole("row")
        .slice(1)
        .map((tr) => tr.textContent ?? "");
    expect(bodyOrder()[0]).toMatch(/charlie/);
    expect(bodyOrder()[1]).toMatch(/alice/);
    expect(bodyOrder()[2]).toMatch(/bob/);

    const nameHeader = screen.getByTestId("tableblock-sort-name");

    // First click → ascending (alice, bob, charlie).
    fireEvent.click(nameHeader);
    expect(bodyOrder()[0]).toMatch(/alice/);
    expect(bodyOrder()[1]).toMatch(/bob/);
    expect(bodyOrder()[2]).toMatch(/charlie/);
    expect(nameHeader.closest("th")).toHaveAttribute(
      "aria-sort",
      "ascending",
    );

    // Second click → descending (charlie, bob, alice).
    fireEvent.click(nameHeader);
    expect(bodyOrder()[0]).toMatch(/charlie/);
    expect(bodyOrder()[1]).toMatch(/bob/);
    expect(bodyOrder()[2]).toMatch(/alice/);
    expect(nameHeader.closest("th")).toHaveAttribute(
      "aria-sort",
      "descending",
    );

    // Third click → reset to server order.
    fireEvent.click(nameHeader);
    expect(bodyOrder()[0]).toMatch(/charlie/);
    expect(bodyOrder()[1]).toMatch(/alice/);
    expect(bodyOrder()[2]).toMatch(/bob/);
    expect(nameHeader.closest("th")).toHaveAttribute("aria-sort", "none");
  });

  it("Phase E polish — numeric sort respects sign and keeps nulls at the bottom", () => {
    wrap(
      make([
        {
          kind: "table",
          columns: [
            { key: "name", label: "Name", align: "left" },
            { key: "score", label: "Score", align: "right" },
          ],
          rows: [
            { name: "alice", score: 10 },
            { name: "charlie", score: null },
            { name: "bob", score: -3 },
            { name: "dave", score: 100 },
          ],
        },
      ]),
    );

    const scoreHeader = screen.getByTestId("tableblock-sort-score");
    fireEvent.click(scoreHeader); // ascending

    const bodyOrder = () =>
      screen
        .getAllByRole("row")
        .slice(1)
        .map((tr) => tr.textContent ?? "");
    // Ascending on a numeric column: -3 < 10 < 100 < null.
    expect(bodyOrder()[0]).toMatch(/bob/);
    expect(bodyOrder()[1]).toMatch(/alice/);
    expect(bodyOrder()[2]).toMatch(/dave/);
    expect(bodyOrder()[3]).toMatch(/charlie/);
  });

  it("renders a badge list with tones", () => {
    wrap(
      make([
        {
          kind: "badge_list",
          items: [
            { label: "active", tone: "ok" },
            { label: "stale", tone: "warn" },
            { label: "crashed", tone: "danger" },
            { label: "neutral", tone: "neutral" },
          ],
        },
      ]),
    );
    expect(screen.getByText("active")).toBeInTheDocument();
    expect(screen.getByText("stale")).toBeInTheDocument();
    expect(screen.getByText("crashed")).toBeInTheDocument();
    expect(screen.getByText("neutral")).toBeInTheDocument();
  });

  it("renders a route button", () => {
    wrap(
      make([
        {
          kind: "button",
          label: "Aller",
          action: { kind: "route", path: "/projects" },
          tone: "neutral",
        },
      ]),
    );
    expect(screen.getByRole("button", { name: "Aller" })).toBeInTheDocument();
  });

  it("renders a task_submit button", () => {
    wrap(
      make([
        {
          kind: "button",
          label: "Lancer",
          action: {
            kind: "task_submit",
            worker: "contradiction_detector",
            payload: null,
          },
          tone: "warn",
        },
      ]),
    );
    expect(screen.getByRole("button", { name: "Lancer" })).toBeInTheDocument();
  });

  it("renders a line chart SVG", () => {
    wrap(
      make([
        {
          kind: "chart_line",
          label: "7 jours",
          points: [
            { x: "lun", y: 1.5 },
            { x: "mar", y: 2.5 },
            { x: "mer", y: 1 },
          ],
          y_unit: "req",
        },
      ]),
    );
    expect(screen.getByText("7 jours")).toBeInTheDocument();
    expect(screen.getByText("req")).toBeInTheDocument();
    expect(
      screen.getByRole("img", { name: "7 jours line chart" }),
    ).toBeInTheDocument();
  });

  it("renders an empty line chart placeholder", () => {
    wrap(make([{ kind: "chart_line", label: "Empty", points: [] }]));
    expect(screen.getByText(/aucun point/)).toBeInTheDocument();
  });

  it("renders a bar chart SVG", () => {
    wrap(
      make([
        {
          kind: "chart_bar",
          label: "Top 3",
          bars: [
            { label: "a", value: 10, tone: "ok" },
            { label: "b", value: 7, tone: "warn" },
            { label: "c", value: 3, tone: "danger" },
          ],
        },
      ]),
    );
    expect(screen.getByText("Top 3")).toBeInTheDocument();
    expect(
      screen.getByRole("img", { name: "Top 3 bar chart" }),
    ).toBeInTheDocument();
  });

  it("renders an empty bar chart placeholder", () => {
    wrap(make([{ kind: "chart_bar", label: "Vide", bars: [] }]));
    expect(screen.getByText(/aucune barre/)).toBeInTheDocument();
  });

  it("renders an empty block", () => {
    wrap(make([{ kind: "empty", text: "nothing to see" }]));
    expect(screen.getByText("nothing to see")).toBeInTheDocument();
  });

  it("renders a section block recursively", () => {
    wrap(
      make([
        {
          kind: "section",
          title: "Outer",
          blocks: [
            {
              kind: "section",
              title: "Inner",
              blocks: [{ kind: "empty", text: "deep" }],
            },
          ],
        },
      ]),
    );
    expect(screen.getByText("Outer")).toBeInTheDocument();
    expect(screen.getByText("Inner")).toBeInTheDocument();
    expect(screen.getByText("deep")).toBeInTheDocument();
  });

  it("renders an empty section placeholder", () => {
    wrap(
      make([
        { kind: "section", title: "Empty section", blocks: [] },
      ]),
    );
    expect(screen.getByText(/section vide/)).toBeInTheDocument();
  });
});

// ---------------------------------------------------------------------------
// Sprint 8 Phase A — ButtonBlock task_submit wiring (D1 + D4)
// ---------------------------------------------------------------------------

// Mock the coordinator API module so the `submitAppTask` call
// doesn't try to hit a real coordinator. The first two tests
// assert the context-less error path and the wired happy path.
vi.mock("@/api/coordinator", () => ({
  submitAppTask: vi.fn(),
}));

import { submitAppTask } from "@/api/coordinator";

const submitAppTaskMock = submitAppTask as unknown as ReturnType<typeof vi.fn>;

beforeEach(() => {
  submitAppTaskMock.mockReset();
});

afterEach(() => {
  submitAppTaskMock.mockReset();
});

describe("ButtonBlock task_submit wiring (Sprint 8 Phase A)", () => {
  function renderButton(options: { withContext: boolean }) {
    const tabView = make([
      {
        kind: "button",
        label: "Lancer",
        action: {
          kind: "task_submit",
          worker: "contradiction_detector",
          payload: { query: "hello" },
        },
        tone: "warn",
      },
    ]);
    const tree = <TabViewRenderer tabView={tabView} />;
    return render(
      <MemoryRouter>
        {options.withContext ? (
          <TabAppContext.Provider
            value={{ coordinatorUrl: "http://127.0.0.1:8765", appName: "gov" }}
          >
            {tree}
          </TabAppContext.Provider>
        ) : (
          tree
        )}
      </MemoryRouter>,
    );
  }

  it("disables the task_submit button when rendered without TabAppContext", () => {
    renderButton({ withContext: false });
    const button = screen.getByRole("button", { name: "Lancer" });
    // The ButtonBlock derives `disabled = (task_submit && tabApp
    // === null)` so a stray click on a badly-placed tab renderer
    // cannot fire submitAppTask with a null coordinator URL.
    expect(button).toBeDisabled();
    expect(submitAppTaskMock).not.toHaveBeenCalled();
  });

  it("calls submitAppTask with the wired coordinator + app + payload", async () => {
    submitAppTaskMock.mockResolvedValue({ task_id: "task-123" });
    renderButton({ withContext: true });
    const button = screen.getByRole("button", { name: "Lancer" });
    fireEvent.click(button);
    await waitFor(() => {
      expect(submitAppTaskMock).toHaveBeenCalledTimes(1);
    });
    expect(submitAppTaskMock).toHaveBeenCalledWith(
      "http://127.0.0.1:8765",
      "gov",
      {
        worker: "contradiction_detector",
        payload: { query: "hello" },
        priority: 5,
        parent_task_id: null,
      },
    );
    await waitFor(() => {
      expect(screen.getByText(/Tâche soumise \(task-123\)/)).toBeInTheDocument();
    });
  });

  it("surfaces submit_task HTTP errors as inline destructive feedback", async () => {
    submitAppTaskMock.mockRejectedValue(new Error("HTTP 422 ghost worker"));
    renderButton({ withContext: true });
    fireEvent.click(screen.getByRole("button", { name: "Lancer" }));
    await waitFor(() => {
      expect(screen.getByText(/HTTP 422 ghost worker/)).toBeInTheDocument();
    });
  });
});

describe("parseTabView + TabViewSchema", () => {
  it("parseTabView accepts a valid payload", () => {
    const result = parseTabView({
      schema_version: 1,
      tab_name: "t",
      blocks: [{ kind: "heading", level: 1, text: "ok" }],
    });
    expect(result.ok).toBe(true);
  });

  it("parseTabView rejects wrong schema_version", () => {
    const result = parseTabView({
      schema_version: 99,
      tab_name: "t",
      blocks: [],
    });
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error).toContain("schema_version");
    }
  });

  it("parseTabView accepts schema_version 2", () => {
    const result = parseTabView({
      schema_version: 2,
      tab_name: "t",
      blocks: [
        { kind: "file_upload", label: "Upload", accept: ["image/*"], max_size_bytes: 1024 },
      ],
    });
    expect(result.ok).toBe(true);
  });

  it("parseTabView rejects unknown block kind", () => {
    const result = parseTabView({
      schema_version: 1,
      tab_name: "t",
      blocks: [{ kind: "unknown", text: "x" }],
    });
    expect(result.ok).toBe(false);
  });

  it("parseTabView rejects missing required field", () => {
    const result = parseTabView({
      schema_version: 1,
      tab_name: "t",
      blocks: [{ kind: "heading", text: "no level" }],
    });
    expect(result.ok).toBe(false);
  });

  it("TabViewSchema accepts blocks: undefined (defaults to [])", () => {
    const parsed = TabViewSchema.safeParse({
      schema_version: 1,
      tab_name: "t",
    });
    expect(parsed.success).toBe(true);
    if (parsed.success) {
      expect(parsed.data.blocks).toEqual([]);
    }
  });
});
