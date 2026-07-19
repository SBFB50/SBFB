// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 83 Phase B — the host→app accessibility projection is needs-free.
 *
 * These tests are the web mirror of the Rust
 * `accessibility_profile::tests::needs_never_serialized_outward`: the operator's
 * needs drive resolution but never reach the DOM, the wire, or a strict parse.
 */
import { readFileSync } from "node:fs";
import { resolve as pathResolve } from "node:path";

import { describe, expect, it } from "vitest";
import {
  ACCESSIBILITY_NEEDS,
  A11Y_PROFILE_EVENT,
  applyResolvedAxes,
  ASSISTIVE_TECHS,
  buildAccessibilityProfileEvent,
  CAPTIONS,
  COLOR_VISIONS,
  DEFAULT_ACCESSIBILITY_PREFERENCES,
  DENSITIES,
  FOCUS_MODES,
  FONTS,
  parseProjectedProfile,
  POINTERS,
  pushAccessibilityProfile,
  READINGS,
  RESOLVED_CONTRASTS,
  RESOLVED_MOTIONS,
  RESOLVED_PROFILE_AXES,
  RESOLVED_THEMES,
  ResolvedAccessibilityProfileSchema,
  resolveAccessibilityProfile,
  SCALES,
  SHORTCUTS,
  TEXT_SPACINGS,
  TRANSPARENCIES,
  type AccessibilityPreferences,
} from "../profile";

function prefsWith(overrides: Partial<AccessibilityPreferences>): AccessibilityPreferences {
  return { ...DEFAULT_ACCESSIBILITY_PREFERENCES, ...overrides };
}

describe("resolveAccessibilityProfile — faithful port", () => {
  it("resolves base preferences to a standard dark profile", () => {
    const p = resolveAccessibilityProfile(DEFAULT_ACCESSIBILITY_PREFERENCES);
    expect(p.theme).toBe("dark");
    expect(p.contrast).toBe("standard");
    expect(p.motion).toBe("standard");
    expect(p.scale).toBe("100");
  });

  it("low vision -> forced theme, high contrast, large pointer, scale>=125", () => {
    const p = resolveAccessibilityProfile(prefsWith({ needs: ["lowVision"] }));
    expect(p.theme).toBe("forced");
    expect(p.contrast).toBe("high");
    expect(p.pointer).toBe("large");
    expect(p.scale).toBe("125");
    expect(p.focus).toBe("strong");
  });

  it("dyslexia -> legible font, loose spacing, assist reading, scale>=112, paper theme", () => {
    const p = resolveAccessibilityProfile(prefsWith({ needs: ["dyslexia"] }));
    expect(p.font).toBe("legible");
    expect(p.textSpacing).toBe("loose");
    expect(p.reading).toBe("assist");
    expect(p.scale).toBe("112");
    expect(p.theme).toBe("paper");
  });

  it("vestibular -> reduced motion + reduced transparency + calm theme (effect, not cause)", () => {
    const p = resolveAccessibilityProfile(prefsWith({ needs: ["vestibular"] }));
    expect(p.motion).toBe("reduced");
    expect(p.transparency).toBe("reduced");
    expect(p.theme).toBe("calm");
  });

  it("system reduced-motion folds in with no needs", () => {
    const p = resolveAccessibilityProfile(DEFAULT_ACCESSIBILITY_PREFERENCES, { reducedMotion: true });
    expect(p.motion).toBe("reduced");
  });
});

describe("needs never crosses host->app", () => {
  it("the resolved profile carries exactly the 15 effect axes and no needs key", () => {
    const p = resolveAccessibilityProfile(prefsWith({ needs: [...ACCESSIBILITY_NEEDS] }));
    const keys = Object.keys(p).sort();
    expect(keys).toEqual([...RESOLVED_PROFILE_AXES].sort());
    expect(keys).not.toContain("needs");
  });

  it("serialising a profile built from ALL needs leaks neither the key nor any need as a value", () => {
    const p = resolveAccessibilityProfile(prefsWith({ needs: [...ACCESSIBILITY_NEEDS] }));
    const json = JSON.stringify(p);
    expect(json).not.toContain("\"needs\"");
    const values = new Set(Object.values(p));
    for (const need of ACCESSIBILITY_NEEDS) {
      expect(values.has(need as never)).toBe(false);
    }
  });

  it("applyResolvedAxes writes the 15 data-* axes but NEVER data-needs", () => {
    const root = document.createElement("div");
    const p = resolveAccessibilityProfile(prefsWith({ needs: [...ACCESSIBILITY_NEEDS] }));
    applyResolvedAxes(p, root);

    // The 15 effect axes are present.
    expect(root.getAttribute("data-theme")).toBe(p.theme);
    expect(root.getAttribute("data-motion")).toBe(p.motion);
    expect(root.getAttribute("data-assistive-tech")).toBe(p.assistiveTech);

    // The cause never reaches the DOM.
    expect(root.getAttribute("data-needs")).toBeNull();
    expect(root.outerHTML).not.toContain("data-needs");
    expect(root.outerHTML).not.toContain("needs");
    // No need identifier leaks as an attribute value either.
    for (const need of ACCESSIBILITY_NEEDS) {
      expect(root.outerHTML).not.toContain(need);
    }
  });

  it("maps each of the 15 axes to its OWN data-* attribute (kills a mapping swap)", () => {
    const root = document.createElement("div");
    // An ALL-DISTINCT-valued projection: every axis holds a value no other
    // axis shares, so wiring an attribute off the wrong axis (e.g.
    // data-transparency off `motion`) is observable. The previous
    // resolve()-based profile let colorVision/motion/transparency collide on
    // the same value and a swap between them survived (Codex S83-B P2). Parsed
    // through the strict schema so it is a real 15-axis projection.
    const p = ResolvedAccessibilityProfileSchema.parse({
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
    });
    applyResolvedAxes(p, root);
    // Wiring an attribute off the wrong axis (e.g. data-scale off `shortcuts`)
    // is observable here (Codex S83-B P2).
    const MAP: Array<[string, keyof typeof p]> = [
      ["data-theme", "theme"],
      ["data-contrast", "contrast"],
      ["data-color-vision", "colorVision"],
      ["data-pointer", "pointer"],
      ["data-text-spacing", "textSpacing"],
      ["data-font", "font"],
      ["data-motion", "motion"],
      ["data-transparency", "transparency"],
      ["data-density", "density"],
      ["data-reading", "reading"],
      ["data-focus", "focus"],
      ["data-scale", "scale"],
      ["data-shortcuts", "shortcuts"],
      ["data-assistive-tech", "assistiveTech"],
      ["data-captions", "captions"],
    ];
    for (const [attr, prop] of MAP) expect(root.getAttribute(attr)).toBe(p[prop]);
  });
});

describe("strict projection schema", () => {
  it("accepts a well-formed projection", () => {
    const p = resolveAccessibilityProfile(prefsWith({ needs: ["lowVision"] }));
    expect(parseProjectedProfile(p)).not.toBeNull();
    expect(ResolvedAccessibilityProfileSchema.safeParse(p).success).toBe(true);
  });

  it("REJECTS a payload that smuggles needs (.strict())", () => {
    const p = resolveAccessibilityProfile(prefsWith({ needs: ["lowVision"] }));
    const smuggled = { ...p, needs: ["lowVision"] };
    expect(parseProjectedProfile(smuggled)).toBeNull();
    expect(ResolvedAccessibilityProfileSchema.safeParse(smuggled).success).toBe(false);
  });

  it("REJECTS a payload with any unknown extra key", () => {
    const p = resolveAccessibilityProfile(DEFAULT_ACCESSIBILITY_PREFERENCES);
    expect(parseProjectedProfile({ ...p, cause: "vestibular" })).toBeNull();
  });

  it("exposes a stable bridge event name", () => {
    expect(A11Y_PROFILE_EVENT).toBe("a11y-profile");
  });
});

describe("host-side sender", () => {
  it("builds a needs-free bridge event even from all-needs preferences", () => {
    const event = buildAccessibilityProfileEvent(prefsWith({ needs: [...ACCESSIBILITY_NEEDS] }));
    expect(event.type).toBe("sbfb-bridge-event");
    expect(event.name).toBe(A11Y_PROFILE_EVENT);
    // The payload passes the strict schema (so it has exactly the 15 axes).
    expect(parseProjectedProfile(event.payload)).not.toBeNull();
    const json = JSON.stringify(event.payload);
    expect(json).not.toContain("\"needs\"");
    for (const need of ACCESSIBILITY_NEEDS) {
      expect(new Set(Object.values(event.payload)).has(need as never)).toBe(false);
    }
  });

  it("posts the needs-free event with an explicit targetOrigin '*'", () => {
    const posted: Array<{ msg: unknown; targetOrigin: unknown }> = [];
    const target = {
      postMessage: (msg: unknown, targetOrigin?: unknown) => posted.push({ msg, targetOrigin }),
    };
    pushAccessibilityProfile(target, prefsWith({ needs: ["lowVision", "cognitive"] }));
    expect(posted).toHaveLength(1);
    // The Hub runs from an OPAQUE sandbox origin (no allow-same-origin), so the
    // host MUST pass targetOrigin "*"; dropping it silently breaks delivery.
    expect(posted[0].targetOrigin).toBe("*");
    const msg = posted[0].msg as { name: string; payload: unknown };
    expect(msg.name).toBe(A11Y_PROFILE_EVENT);
    expect(parseProjectedProfile(msg.payload)).not.toBeNull();
    expect(JSON.stringify(msg)).not.toContain("needs");
  });
});

describe("cross-implementation parity oracle (shared fixture)", () => {
  // The SAME fixture the Rust `accessibility_profile::tests::resolution_fixtures`
  // consumes: needs + system -> the exact 15-axis projection. If TS resolve
  // diverges from the Rust-generated oracle (or the oracle drifts), this fails.
  // Proves Rust<->TS parity for every isolated need, the OS signals, stacking
  // combinations and the all-needs case (Codex S83-B P2).
  // vitest cwd is the `web/` dir; the fixture lives one level up in `crates/`.
  const fixturesPath = pathResolve(
    process.cwd(),
    "../crates/sbfb-workflow-core/src/accessibility_fixtures.json",
  );
  const fixtures = JSON.parse(readFileSync(fixturesPath, "utf8")) as {
    cases: Array<{
      name: string;
      prefs: AccessibilityPreferences;
      system: { reducedMotion: boolean; highContrast: boolean; forcedColors: boolean };
      expected: Record<string, string>;
    }>;
  };

  it("has at least the 13 isolated-need cases plus system + stacking + pref cases", () => {
    expect(fixtures.cases.length).toBeGreaterThanOrEqual(ACCESSIBILITY_NEEDS.length + 6);
  });

  for (const c of fixtures.cases) {
    it(`resolves case '${c.name}' identically to the Rust oracle`, () => {
      const resolved = resolveAccessibilityProfile(c.prefs, {
        reducedMotion: c.system.reducedMotion,
        highContrast: c.system.highContrast,
        forcedColors: c.system.forcedColors,
      });
      expect(resolved).toEqual(c.expected);
      // The oracle itself must be needs-free.
      expect(c.expected).not.toHaveProperty("needs");
    });
  }
});

describe("golden-schema vocabulary parity (Codex S83-B P2 #9)", () => {
  // The Rust-generated golden schema is the AUTHORITATIVE vocabulary of the 15
  // resolved axes. The fixture oracle above proves resolution parity but NOT
  // that the value vocabularies agree: adding e.g. `scale="175"` to the TS const
  // only (never regenerating the Rust golden) would leave every resolution test
  // green while diverging from Rust/CSS. This locks each axis's value set exactly
  // against the golden, so a one-sided vocabulary edit fails the suite.
  const schema = JSON.parse(
    readFileSync(
      pathResolve(process.cwd(), "../crates/sbfb-workflow-core/src/accessibility_profile.schema.json"),
      "utf8",
    ),
  ) as {
    properties: Record<string, { $ref: string }>;
    $defs: Record<string, { enum: string[] }>;
  };
  const goldenEnum = (axis: string): string[] => {
    const def = schema.properties[axis].$ref.split("/").pop() as string;
    return schema.$defs[def].enum;
  };

  const AXIS_VOCAB: Record<string, readonly string[]> = {
    theme: RESOLVED_THEMES,
    contrast: RESOLVED_CONTRASTS,
    colorVision: COLOR_VISIONS,
    pointer: POINTERS,
    textSpacing: TEXT_SPACINGS,
    font: FONTS,
    motion: RESOLVED_MOTIONS,
    transparency: TRANSPARENCIES,
    density: DENSITIES,
    reading: READINGS,
    focus: FOCUS_MODES,
    scale: SCALES,
    shortcuts: SHORTCUTS,
    assistiveTech: ASSISTIVE_TECHS,
    captions: CAPTIONS,
  };

  it("declares exactly the 15 golden axis keys (no TS-only or golden-only axis)", () => {
    const golden = Object.keys(schema.properties).sort();
    expect([...RESOLVED_PROFILE_AXES].sort()).toEqual(golden);
    expect(Object.keys(AXIS_VOCAB).sort()).toEqual(golden);
  });

  for (const [axis, tsVocab] of Object.entries(AXIS_VOCAB)) {
    it(`axis '${axis}' value vocabulary matches the Rust golden enum exactly`, () => {
      expect([...tsVocab].sort()).toEqual([...goldenEnum(axis)].sort());
    });
  }
});
