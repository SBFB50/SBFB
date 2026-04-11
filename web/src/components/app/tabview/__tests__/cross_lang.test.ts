/**
 * Sprint 6 audit fix A-3 — cross-language TabView contract guard.
 *
 * Pair test to `packages/nexus-sdk/tests/test_view.py`'s
 * `test_canonical_fixture_roundtrip`. Both tests load the SAME
 * JSON fixture at
 * `packages/nexus-sdk/tests/snapshots/tabview_canonical.json`.
 *
 * The Python side validates via `TabView.model_validate` and
 * asserts a lossless `model_dump()` round-trip. This side parses
 * via `TabViewSchema.safeParse` and asserts success + structural
 * agreement.
 *
 * Before Sprint 6 audit, the only cross-language guard was the
 * docstring of `test_view_schema_stable_snapshot`, which claimed
 * cross-language coverage but only checked the Pydantic side
 * against its own stored schema dump. A rename in `schema.ts`
 * would sail through. This fixture is the real guard: any
 * Python-only or Zod-only change breaks exactly one of these two
 * tests, pinpointing which side drifted.
 */

import { describe, expect, it } from "vitest";

import { TabViewSchema, parseTabView } from "../schema";
// Static JSON import resolved by tsc (resolveJsonModule) AND Vite.
// The same file is read by packages/nexus-sdk/tests/test_view.py
// via pathlib — see test_canonical_fixture_roundtrip there.
// Cross-repo path is intentional: a single fixture is the point.
import canonical from "../../../../../../packages/nexus-sdk/tests/snapshots/tabview_canonical.json";

function loadFixture(): unknown {
  return canonical;
}

describe("cross-language TabView canonical fixture", () => {
  it("fixture file exists at the shared path", () => {
    expect(() => loadFixture()).not.toThrow();
  });

  it("Zod parses the full Python-generated fixture without errors", () => {
    const payload = loadFixture();
    const parsed = TabViewSchema.safeParse(payload);
    if (!parsed.success) {
      // Surface the first issue so a drift is pinpointable.
      const issues = parsed.error.issues
        .slice(0, 5)
        .map((i) => `${i.path.join(".") || "(root)"}: ${i.message}`)
        .join("\n  ");
      throw new Error(
        `Zod rejected the canonical fixture. Cross-language drift.\n` +
          `Check schema.ts against view.py, then regenerate\n` +
          `tabview_canonical.json if the change is intentional.\n\n` +
          `First issues:\n  ${issues}`,
      );
    }
    expect(parsed.success).toBe(true);
  });

  it("parseTabView helper returns ok=true on the fixture", () => {
    const payload = loadFixture();
    const result = parseTabView(payload);
    expect(result.ok).toBe(true);
  });

  it("fixture exercises all 11 block kinds (same invariant as the Python test)", () => {
    const payload = loadFixture() as {
      blocks: Array<Record<string, unknown>>;
    };
    const kinds = new Set<string>();
    function collect(blocks: Array<Record<string, unknown>>): void {
      for (const block of blocks) {
        const kind = block.kind as string | undefined;
        if (typeof kind === "string") {
          kinds.add(kind);
          if (kind === "section") {
            const nested = (block.blocks ?? []) as Array<
              Record<string, unknown>
            >;
            collect(nested);
          }
        }
      }
    }
    collect(payload.blocks);
    expect(kinds).toEqual(
      new Set([
        "section",
        "heading",
        "text",
        "kv",
        "metric",
        "table",
        "badge_list",
        "button",
        "chart_line",
        "chart_bar",
        "empty",
      ]),
    );
  });

  it("fixture preserves non-ASCII characters through Zod parse", () => {
    const parsed = TabViewSchema.parse(loadFixture());
    const flat = JSON.stringify(parsed);
    // Coffee-adjacent smoke check — caféccedillas should survive.
    expect(flat).toContain("café");
    expect(flat).toContain("Ève");
    expect(flat).toContain("tâches");
  });

  it("fixture title is populated (optional field present-case)", () => {
    const parsed = TabViewSchema.parse(loadFixture());
    expect(parsed.title).toBeTruthy();
    expect(parsed.schema_version).toBe(1);
    expect(parsed.tab_name).toBe("canonical");
  });
});
