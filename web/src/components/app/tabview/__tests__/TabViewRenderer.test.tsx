/**
 * Sprint 6 Phase D — TabView renderer tests.
 *
 * One test per block kind (11 total) plus the schema validation
 * edge cases. Tests use @testing-library/react + jsdom. React
 * Router's <MemoryRouter> wraps renders that exercise the
 * <ButtonBlock> path (which calls useNavigate).
 */

import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";

import { TabViewRenderer } from "../TabViewRenderer";
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
      schema_version: 2,
      tab_name: "t",
      blocks: [],
    });
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error).toContain("schema_version");
    }
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
