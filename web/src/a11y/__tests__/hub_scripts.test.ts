// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 83 Phase B — execute the VENDORED Workflow Hub scripts (Codex S83-B).
 *
 * `web/src/a11y/*` is unit-tested, but the Hub archive ships its OWN vendored
 * `preboot.js` + `a11y.js` (a separate origin). This suite reads those exact
 * files and runs them in jsdom, so the needs-free preboot and the strict live
 * receiver are actually exercised — not just their web-module cousins.
 */
import { readFileSync } from "node:fs";
import { resolve as pathResolve } from "node:path";

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const HUB = pathResolve(process.cwd(), "../examples/workflow-hub");
const PREBOOT_SRC = readFileSync(pathResolve(HUB, "preboot.js"), "utf8");
const A11Y_SRC = readFileSync(pathResolve(HUB, "a11y.js"), "utf8");

// The Rust-generated golden schema is the AUTHORITATIVE 15-axis vocabulary.
const GOLDEN = JSON.parse(
  readFileSync(
    pathResolve(process.cwd(), "../crates/sbfb-workflow-core/src/accessibility_profile.schema.json"),
    "utf8",
  ),
) as { properties: Record<string, { $ref: string }>; $defs: Record<string, { enum: string[] }> };
const goldenEnum = (axis: string): string[] => {
  const def = GOLDEN.properties[axis].$ref.split("/").pop() as string;
  return GOLDEN.$defs[def].enum;
};

const AXIS_ATTRS = [
  "data-theme",
  "data-contrast",
  "data-color-vision",
  "data-pointer",
  "data-text-spacing",
  "data-font",
  "data-motion",
  "data-transparency",
  "data-density",
  "data-reading",
  "data-focus",
  "data-scale",
  "data-shortcuts",
  "data-assistive-tech",
  "data-captions",
];

const VALID = {
  theme: "dark",
  contrast: "high",
  colorVision: "safe",
  pointer: "large",
  textSpacing: "loose",
  font: "legible",
  motion: "reduced",
  transparency: "standard",
  density: "focus",
  reading: "assist",
  focus: "strong",
  scale: "125",
  shortcuts: "off",
  assistiveTech: "screen-reader",
  captions: "on",
};

function runScript(src: string) {
  (0, eval)(src);
}

function clearRoot() {
  const root = document.documentElement;
  for (const name of Array.from(root.attributes).map((a) => a.name)) {
    if (name.startsWith("data-")) root.removeAttribute(name);
  }
  root.removeAttribute("style");
}

describe("vendored Hub preboot.js (executed in jsdom)", () => {
  beforeEach(() => {
    clearRoot();
    delete (window as any).SbfbHubA11y;
  });

  it("applies the 15 effect axes and NEVER data-needs, exposing SbfbHubA11y", () => {
    runScript(PREBOOT_SRC);
    const root = document.documentElement;
    for (const attr of AXIS_ATTRS) expect(root.hasAttribute(attr)).toBe(true);
    expect(root.hasAttribute("data-needs")).toBe(false);
    expect(root.outerHTML).not.toContain("data-needs");
    expect((window as any).SbfbHubA11y?.applyProfile).toBeTypeOf("function");
  });
});

describe("vendored Hub preboot.js — OS signal fallbacks (executed in jsdom)", () => {
  const origMatchMedia = window.matchMedia;
  beforeEach(() => {
    clearRoot();
    delete (window as any).SbfbHubA11y;
  });
  afterEach(() => {
    (window as any).matchMedia = origMatchMedia;
  });
  // Mock matchMedia so ONLY the listed media features report `matches: true`.
  function mockMatchMedia(matching: string[]) {
    (window as any).matchMedia = (q: string) => ({
      matches: matching.some((m) => q.includes(m)),
      media: q,
    });
  }

  it("folds in prefers-reduced-motion -> data-motion=reduced (dropping the fallback is caught)", () => {
    mockMatchMedia(["prefers-reduced-motion"]);
    runScript(PREBOOT_SRC);
    expect(document.documentElement.getAttribute("data-motion")).toBe("reduced");
  });

  it("folds in prefers-contrast:more -> data-contrast=high", () => {
    mockMatchMedia(["prefers-contrast"]);
    runScript(PREBOOT_SRC);
    expect(document.documentElement.getAttribute("data-contrast")).toBe("high");
  });

  it("folds in forced-colors -> data-theme=forced + data-contrast=high", () => {
    mockMatchMedia(["forced-colors"]);
    runScript(PREBOOT_SRC);
    expect(document.documentElement.getAttribute("data-theme")).toBe("forced");
    expect(document.documentElement.getAttribute("data-contrast")).toBe("high");
  });

  it("no OS signal -> needs-free defaults (dark / standard / standard)", () => {
    mockMatchMedia([]);
    runScript(PREBOOT_SRC);
    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
    expect(document.documentElement.getAttribute("data-contrast")).toBe("standard");
    expect(document.documentElement.getAttribute("data-motion")).toBe("standard");
  });
});

describe("vendored Hub a11y.js (executed, live channel)", () => {
  let handler: ((e: any) => void) | null;
  let fakeParent: { postMessage: ReturnType<typeof vi.fn> };
  let origParent: PropertyDescriptor | undefined;

  beforeEach(() => {
    clearRoot();
    delete (window as any).SbfbHubA11y;
    runScript(PREBOOT_SRC);

    handler = null;
    fakeParent = { postMessage: vi.fn() };
    origParent = Object.getOwnPropertyDescriptor(window, "parent");
    Object.defineProperty(window, "parent", { configurable: true, value: fakeParent });

    const origAdd = window.addEventListener.bind(window);
    const spy = vi
      .spyOn(window, "addEventListener")
      .mockImplementation((type: any, h: any, opts?: any) => {
        if (type === "message") handler = h;
        else origAdd(type, h, opts);
      });
    runScript(A11Y_SRC);
    spy.mockRestore();
  });

  afterEach(() => {
    if (origParent) Object.defineProperty(window, "parent", origParent);
  });

  function send(payload: unknown, source: unknown = fakeParent) {
    handler?.({ data: { type: "sbfb-bridge-event", name: "a11y-profile", payload }, source });
  }

  it("registers a message listener and announces readiness to the parent", () => {
    expect(handler).toBeTypeOf("function");
    expect(fakeParent.postMessage).toHaveBeenCalled();
  });

  it("applies a valid 15-axis profile from the parent (needs-free)", () => {
    send(VALID);
    const root = document.documentElement;
    expect(root.getAttribute("data-motion")).toBe("reduced");
    expect(root.getAttribute("data-assistive-tech")).toBe("screen-reader");
    expect(root.hasAttribute("data-needs")).toBe(false);
    expect(root.outerHTML).not.toContain("data-needs");
  });

  it("maps EACH of the 15 axes to its OWN data-* attribute (kills a mapping swap)", () => {
    send(VALID);
    const root = document.documentElement;
    // Distinct-valued VALID profile: e.g. wiring data-scale off `shortcuts`
    // would yield data-scale="off" != "125" and fail here.
    const MAP: Array<[string, string]> = [
      ["data-theme", VALID.theme],
      ["data-contrast", VALID.contrast],
      ["data-color-vision", VALID.colorVision],
      ["data-pointer", VALID.pointer],
      ["data-text-spacing", VALID.textSpacing],
      ["data-font", VALID.font],
      ["data-motion", VALID.motion],
      ["data-transparency", VALID.transparency],
      ["data-density", VALID.density],
      ["data-reading", VALID.reading],
      ["data-focus", VALID.focus],
      ["data-scale", VALID.scale],
      ["data-shortcuts", VALID.shortcuts],
      ["data-assistive-tech", VALID.assistiveTech],
      ["data-captions", VALID.captions],
    ];
    for (const [attr, value] of MAP) expect(root.getAttribute(attr)).toBe(value);
  });

  it("REJECTS a payload smuggling needs (extra key)", () => {
    document.documentElement.setAttribute("data-motion", "standard");
    send({ ...VALID, needs: ["lowVision"] });
    expect(document.documentElement.getAttribute("data-motion")).toBe("standard");
    expect(document.documentElement.hasAttribute("data-needs")).toBe(false);
  });

  it("REJECTS an incomplete payload (missing axes)", () => {
    document.documentElement.setAttribute("data-motion", "standard");
    send({ motion: "reduced" });
    expect(document.documentElement.getAttribute("data-motion")).toBe("standard");
  });

  it("REJECTS an invalid axis value", () => {
    document.documentElement.setAttribute("data-scale", "100");
    send({ ...VALID, scale: "999" });
    expect(document.documentElement.getAttribute("data-scale")).toBe("100");
  });

  it("REJECTS a message whose source is not the parent (null AND a distinct non-parent window)", () => {
    document.documentElement.setAttribute("data-motion", "standard");
    // null source
    send(VALID, null);
    expect(document.documentElement.getAttribute("data-motion")).toBe("standard");
    // A distinct, NON-null source that is not window.parent — a weakened mutant
    // `fromParent = !!event.source` would WRONGLY accept these (Codex S83-B P2).
    send(VALID, { postMessage() {} });
    expect(document.documentElement.getAttribute("data-motion")).toBe("standard");
    // window itself (self / top-level), which is not the parent, is also rejected.
    send(VALID, window);
    expect(document.documentElement.getAttribute("data-motion")).toBe("standard");
  });
});

describe("vendored Hub AXES vocabulary parity with the Rust golden (Codex S83-B P2 #9)", () => {
  beforeEach(() => {
    clearRoot();
    delete (window as any).SbfbHubA11y;
  });

  it("SbfbHubA11y.AXES declares exactly the 15 golden axes, each with the golden enum", () => {
    runScript(PREBOOT_SRC);
    const axes = (window as any).SbfbHubA11y.AXES as Record<string, string[]>;
    const goldenKeys = Object.keys(GOLDEN.properties).sort();
    // No Hub-only axis and no golden axis the Hub forgot.
    expect(Object.keys(axes).sort()).toEqual(goldenKeys);
    // Each axis' value set matches the Rust golden exactly — a one-sided edit
    // (e.g. adding `scale='175'` to the vendored preboot only) fails here.
    for (const axis of goldenKeys) {
      expect([...axes[axis]].sort()).toEqual([...goldenEnum(axis)].sort());
    }
  });
});

describe("embedded mode-guide projections parity with mode-guide.json (Codex S83-B P3)", () => {
  // index.html embeds a copy of the mode-guide projections (read without fetch,
  // CSP connect-src 'none'). That embedded copy is a SECOND source: it must not
  // silently diverge from the canonical mode-guide.json. This locks id/label/
  // summary equality so an edit to one artifact without the other fails.
  it("index.html #mode-guide-data projections equal mode-guide.json on id/label/summary", () => {
    const html = readFileSync(pathResolve(HUB, "index.html"), "utf8");
    const m = html.match(
      /<script type="application\/json" id="mode-guide-data">([\s\S]*?)<\/script>/,
    );
    expect(m).not.toBeNull();
    const embedded = JSON.parse((m as RegExpMatchArray)[1]) as {
      projections: Array<{ id: string; label: string; summary: string }>;
    };
    const source = JSON.parse(readFileSync(pathResolve(HUB, "mode-guide.json"), "utf8")) as {
      projections: Array<{ id: string; label: string; summary: string }>;
    };
    const pick = (p: { id: string; label: string; summary: string }) => ({
      id: p.id,
      label: p.label,
      summary: p.summary,
    });
    expect(embedded.projections.map(pick)).toEqual(source.projections.map(pick));
  });
});
