// SPDX-License-Identifier: AGPL-3.0-or-later
/**
 * Sprint 83 Phase B — shared host→app accessibility module.
 *
 * The TypeScript mirror of `ResolvedAccessibilityProfile.v1`
 * (`crates/sbfb-workflow-core/src/accessibility_profile.rs`): the needs-free
 * profile the host projects to a sandboxed SBFB app (the Workflow Hub). This
 * module is the SINGLE authoritative source for the shell host side; the Hub
 * archive vendors a byte-equivalent needs-free applier for its own bundle.
 *
 * ## Cardinal invariant — `needs[]` never crosses host→app
 *
 * The operator's {@link AccessibilityNeed}s are health/disability data. They
 * drive resolution LOCALLY (host-side) then are dropped: the projection type
 * {@link ResolvedAccessibilityProfile} carries only the resolved EFFECT axes
 * (`motion: 'reduced'`, `contrast: 'high'`, …), never the cause. The invariant
 * holds by construction:
 *   - the type has no `needs` field;
 *   - {@link ResolvedAccessibilityProfileSchema} is `.strict()`, so a payload
 *     carrying `needs` is REJECTED on the app side;
 *   - {@link applyResolvedAxes} writes the 15 effect `data-*` attributes and
 *     NEVER `data-needs` (unlike the reference host tool
 *     `tools/factory-operator/public/preboot.js:367`, which does — acceptable
 *     for a first-party privileged host tool writing its OWN local DOM under
 *     its own CSP, unacceptable for an untrusted sandboxed app).
 *
 * This module NEVER imports the Factory Operator (§20.7): the Hub is a
 * sandboxed iframe app, the Operator a privileged host tool; they must not
 * cross. The resolution logic is re-derived here axis for axis so the two
 * agree.
 */

import { z } from "zod";

// =================================================================
// Axis vocabularies — mirror the Rust enums (kebab-case values).
// =================================================================

/** The 13 accessibility needs (the cause). Host-local; never projected. */
export const ACCESSIBILITY_NEEDS = [
  "lowVision",
  "blindAssistive",
  "colorVision",
  "noColor",
  "vestibular",
  "photosensitive",
  "motor",
  "cognitive",
  "dyslexia",
  "attention",
  "auditory",
  "speech",
  "sensory",
] as const;
export type AccessibilityNeed = (typeof ACCESSIBILITY_NEEDS)[number];

export const THEMES = ["auto", "dark", "light", "calm", "paper", "forced"] as const;
export const RESOLVED_THEMES = ["dark", "light", "calm", "paper", "forced"] as const;
export const CONTRASTS = ["auto", "standard", "high"] as const;
export const RESOLVED_CONTRASTS = ["standard", "high"] as const;
export const MOTIONS = ["system", "reduced"] as const;
export const RESOLVED_MOTIONS = ["standard", "reduced"] as const;
export const COLOR_VISIONS = ["standard", "safe", "monochrome"] as const;
export const POINTERS = ["standard", "large"] as const;
export const TEXT_SPACINGS = ["standard", "loose"] as const;
export const FONTS = ["standard", "legible"] as const;
export const TRANSPARENCIES = ["standard", "reduced"] as const;
export const DENSITIES = ["standard", "focus"] as const;
export const READINGS = ["standard", "assist"] as const;
export const FOCUS_MODES = ["standard", "strong"] as const;
export const SCALES = ["100", "112", "125", "150"] as const;
export const SHORTCUTS = ["off", "on"] as const;
export const ASSISTIVE_TECHS = ["standard", "screen-reader"] as const;
export const CAPTIONS = ["off", "on"] as const;

export type ThemePreference = (typeof THEMES)[number];
export type ResolvedTheme = (typeof RESOLVED_THEMES)[number];
export type ContrastPreference = (typeof CONTRASTS)[number];
export type ResolvedContrast = (typeof RESOLVED_CONTRASTS)[number];
export type MotionPreference = (typeof MOTIONS)[number];
export type ResolvedMotion = (typeof RESOLVED_MOTIONS)[number];
export type ColorVision = (typeof COLOR_VISIONS)[number];
export type Pointer = (typeof POINTERS)[number];
export type TextSpacing = (typeof TEXT_SPACINGS)[number];
export type Font = (typeof FONTS)[number];
export type Transparency = (typeof TRANSPARENCIES)[number];
export type Density = (typeof DENSITIES)[number];
export type Reading = (typeof READINGS)[number];
export type Focus = (typeof FOCUS_MODES)[number];
export type Scale = (typeof SCALES)[number];
export type Shortcut = (typeof SHORTCUTS)[number];
export type AssistiveTech = (typeof ASSISTIVE_TECHS)[number];
export type Caption = (typeof CAPTIONS)[number];

/**
 * The operator's base preferences plus declared needs — host-local input to
 * {@link resolveAccessibilityProfile}. May be persisted host-side; NEVER sent
 * to an app.
 */
export interface AccessibilityPreferences {
  needs: AccessibilityNeed[];
  theme: ThemePreference;
  contrast: ContrastPreference;
  colorVision: ColorVision;
  pointer: Pointer;
  textSpacing: TextSpacing;
  font: Font;
  motion: MotionPreference;
  transparency: Transparency;
  density: Density;
  reading: Reading;
  focus: Focus;
  scale: Scale;
  shortcuts: Shortcut;
  assistiveTech: AssistiveTech;
  captions: Caption;
}

/** OS/browser-reported accessibility signals folded into resolution. */
export interface AccessibilitySystem {
  reducedMotion?: boolean;
  highContrast?: boolean;
  forcedColors?: boolean;
}

/**
 * `ResolvedAccessibilityProfile.v1` — the needs-free host→app projection.
 * The 15 resolved effect axes and NOTHING else. No `needs` field exists.
 */
export interface ResolvedAccessibilityProfile {
  theme: ResolvedTheme;
  contrast: ResolvedContrast;
  colorVision: ColorVision;
  pointer: Pointer;
  textSpacing: TextSpacing;
  font: Font;
  motion: ResolvedMotion;
  transparency: Transparency;
  density: Density;
  reading: Reading;
  focus: Focus;
  scale: Scale;
  shortcuts: Shortcut;
  assistiveTech: AssistiveTech;
  captions: Caption;
}

/** The 15 effect-axis keys, in schema order. */
export const RESOLVED_PROFILE_AXES = [
  "theme",
  "contrast",
  "colorVision",
  "pointer",
  "textSpacing",
  "font",
  "motion",
  "transparency",
  "density",
  "reading",
  "focus",
  "scale",
  "shortcuts",
  "assistiveTech",
  "captions",
] as const;

/**
 * Strict Zod schema for a projected profile. `.strict()` is load-bearing: a
 * payload that smuggles `needs` (or any other extra key) is REJECTED, so the
 * app never even accepts the cause across the bridge.
 */
export const ResolvedAccessibilityProfileSchema = z
  .object({
    theme: z.enum(RESOLVED_THEMES),
    contrast: z.enum(RESOLVED_CONTRASTS),
    colorVision: z.enum(COLOR_VISIONS),
    pointer: z.enum(POINTERS),
    textSpacing: z.enum(TEXT_SPACINGS),
    font: z.enum(FONTS),
    motion: z.enum(RESOLVED_MOTIONS),
    transparency: z.enum(TRANSPARENCIES),
    density: z.enum(DENSITIES),
    reading: z.enum(READINGS),
    focus: z.enum(FOCUS_MODES),
    scale: z.enum(SCALES),
    shortcuts: z.enum(SHORTCUTS),
    assistiveTech: z.enum(ASSISTIVE_TECHS),
    captions: z.enum(CAPTIONS),
  })
  .strict();

/** The bridge event name carrying a profile from host to a sandboxed app. */
export const A11Y_PROFILE_EVENT = "a11y-profile";

/** The reference-tool default preferences. */
export const DEFAULT_ACCESSIBILITY_PREFERENCES: AccessibilityPreferences = {
  needs: [],
  theme: "auto",
  contrast: "auto",
  colorVision: "standard",
  pointer: "standard",
  textSpacing: "standard",
  font: "standard",
  motion: "system",
  transparency: "standard",
  density: "standard",
  reading: "standard",
  focus: "standard",
  scale: "100",
  shortcuts: "off",
  assistiveTech: "standard",
  captions: "off",
};

// =================================================================
// Resolution — faithful port of accessibility.ts / preboot.js.
// =================================================================

const SCALE_RANK: Record<Scale, number> = { "100": 0, "112": 1, "125": 2, "150": 3 };

function hasNeed(prefs: AccessibilityPreferences, need: AccessibilityNeed): boolean {
  return prefs.needs.includes(need);
}

function hasAnyNeed(prefs: AccessibilityPreferences, needs: readonly AccessibilityNeed[]): boolean {
  return needs.some((n) => hasNeed(prefs, n));
}

function atLeastScale(current: Scale, minimum: Scale): Scale {
  return SCALE_RANK[current] >= SCALE_RANK[minimum] ? current : minimum;
}

function resolveTheme(prefs: AccessibilityPreferences, system: AccessibilitySystem): ResolvedTheme {
  if (prefs.theme !== "auto") return prefs.theme;
  if (system.forcedColors || hasAnyNeed(prefs, ["lowVision", "blindAssistive", "noColor"])) return "forced";
  if (hasAnyNeed(prefs, ["vestibular", "photosensitive", "attention", "sensory"])) return "calm";
  if (hasAnyNeed(prefs, ["cognitive", "dyslexia"])) return "paper";
  return "dark";
}

function resolveContrast(prefs: AccessibilityPreferences, system: AccessibilitySystem): ResolvedContrast {
  if (
    prefs.contrast === "high" ||
    system.highContrast ||
    system.forcedColors ||
    hasAnyNeed(prefs, ["lowVision", "blindAssistive", "colorVision", "noColor"])
  ) {
    return "high";
  }
  return "standard";
}

/**
 * Resolve a needs-free {@link ResolvedAccessibilityProfile} from the
 * operator's preferences and the OS/browser signals. Axis-for-axis port of
 * `resolveAccessibilityPreferences` (`accessibility.ts`) and `resolve`
 * (`preboot.js`); the output deliberately omits `needs`.
 */
export function resolveAccessibilityProfile(
  prefs: AccessibilityPreferences,
  system: AccessibilitySystem = {},
): ResolvedAccessibilityProfile {
  let scale = prefs.scale;
  let pointer = prefs.pointer;
  let textSpacing = prefs.textSpacing;
  let font = prefs.font;
  let colorVision = prefs.colorVision;
  let transparency = prefs.transparency;
  let density = prefs.density;
  let reading = prefs.reading;
  let focus = prefs.focus;
  let shortcuts = prefs.shortcuts;
  let assistiveTech = prefs.assistiveTech;
  let captions = prefs.captions;
  let motion: ResolvedMotion = prefs.motion === "reduced" || system.reducedMotion ? "reduced" : "standard";

  if (hasNeed(prefs, "lowVision")) {
    scale = atLeastScale(scale, "125");
    pointer = "large";
    focus = "strong";
  }
  if (hasNeed(prefs, "blindAssistive")) {
    assistiveTech = "screen-reader";
    density = "focus";
    focus = "strong";
    shortcuts = "off";
  }
  if (hasNeed(prefs, "colorVision")) {
    colorVision = colorVision === "monochrome" ? "monochrome" : "safe";
    focus = "strong";
  }
  if (hasNeed(prefs, "noColor")) {
    colorVision = "monochrome";
    focus = "strong";
  }
  if (hasAnyNeed(prefs, ["vestibular", "photosensitive", "attention", "sensory"])) {
    motion = "reduced";
    transparency = "reduced";
  }
  if (hasNeed(prefs, "motor")) {
    pointer = "large";
    focus = "strong";
    shortcuts = "off";
  }
  if (hasNeed(prefs, "cognitive")) {
    density = "focus";
    reading = "assist";
    textSpacing = "loose";
    shortcuts = "off";
  }
  if (hasNeed(prefs, "dyslexia")) {
    font = "legible";
    textSpacing = "loose";
    reading = "assist";
    scale = atLeastScale(scale, "112");
  }
  if (hasNeed(prefs, "auditory")) captions = "on";
  if (hasNeed(prefs, "speech")) shortcuts = "off";

  return {
    theme: resolveTheme(prefs, system),
    contrast: resolveContrast(prefs, system),
    colorVision,
    pointer,
    textSpacing,
    font,
    motion,
    transparency,
    density,
    reading,
    focus,
    scale,
    shortcuts,
    assistiveTech,
    captions,
  };
}

// =================================================================
// Colour tokens (oklch) — ported from preboot.js, theme-driven.
// =================================================================

const COLOR_KEYS = [
  "s0", "s1", "s2", "s3", "bd", "bd2", "field", "tx", "tx2", "tx3", "tx4",
  "ok", "warn", "bad", "info", "neu", "ok-bg", "bad-bg",
] as const;
type ColorMap = Partial<Record<(typeof COLOR_KEYS)[number], string>>;

const LIGHT_COLORS: ColorMap = {
  s0: "oklch(0.975 0.004 260)", s1: "oklch(0.945 0.005 260)", s2: "oklch(0.910 0.006 260)",
  s3: "oklch(0.870 0.008 260)", bd: "oklch(0.740 0.010 260)", bd2: "oklch(0.610 0.012 260)",
  field: "oklch(0.500 0.014 260)", tx: "oklch(0.185 0.008 260)", tx2: "oklch(0.315 0.010 260)",
  tx3: "oklch(0.405 0.010 260)", tx4: "oklch(0.470 0.010 260)", ok: "oklch(0.420 0.130 150)",
  warn: "oklch(0.470 0.120 78)", bad: "oklch(0.475 0.170 25)", info: "oklch(0.430 0.120 240)",
  neu: "oklch(0.405 0.010 260)", "ok-bg": "oklch(0.910 0.030 150)", "bad-bg": "oklch(0.920 0.035 25)",
};
const CALM_COLORS: ColorMap = {
  s0: "oklch(0.200 0.012 180)", s1: "oklch(0.235 0.012 180)", s2: "oklch(0.275 0.014 180)",
  s3: "oklch(0.320 0.015 180)", bd: "oklch(0.390 0.012 180)", bd2: "oklch(0.500 0.014 180)",
  field: "oklch(0.610 0.016 180)", tx: "oklch(0.930 0.006 170)", tx2: "oklch(0.760 0.008 170)",
  tx3: "oklch(0.700 0.009 170)", tx4: "oklch(0.640 0.010 170)", ok: "oklch(0.760 0.090 155)",
  warn: "oklch(0.800 0.090 80)", bad: "oklch(0.760 0.105 25)", info: "oklch(0.780 0.070 220)",
  neu: "oklch(0.700 0.009 170)", "ok-bg": "oklch(0.310 0.032 155)", "bad-bg": "oklch(0.300 0.035 25)",
};
const PAPER_COLORS: ColorMap = {
  s0: "oklch(0.970 0.006 95)", s1: "oklch(0.945 0.008 95)", s2: "oklch(0.910 0.010 95)",
  s3: "oklch(0.860 0.012 95)", bd: "oklch(0.720 0.014 95)", bd2: "oklch(0.560 0.016 95)",
  field: "oklch(0.410 0.018 95)", tx: "oklch(0.160 0.012 80)", tx2: "oklch(0.290 0.014 80)",
  tx3: "oklch(0.380 0.014 80)", tx4: "oklch(0.455 0.014 80)", ok: "oklch(0.400 0.120 150)",
  warn: "oklch(0.455 0.115 78)", bad: "oklch(0.455 0.160 25)", info: "oklch(0.390 0.110 240)",
  neu: "oklch(0.380 0.014 80)", "ok-bg": "oklch(0.900 0.032 150)", "bad-bg": "oklch(0.905 0.036 25)",
};
const FORCED_COLORS: ColorMap = {
  s0: "#000000", s1: "#050505", s2: "#101010", s3: "#1a1a1a", bd: "#9f9f9f", bd2: "#ffffff",
  field: "#ffffff", tx: "#ffffff", tx2: "#f2f2f2", tx3: "#e6e6e6", tx4: "#d8d8d8", ok: "#ffffff",
  warn: "#ffff00", bad: "#ff6b6b", info: "#66ccff", neu: "#e6e6e6", "ok-bg": "#111111", "bad-bg": "#220000",
};
const HIGH_COLORS: ColorMap = {
  bd: "oklch(0.560 0.010 260)", bd2: "oklch(0.720 0.012 260)", field: "oklch(0.760 0.014 260)",
  tx: "oklch(0.985 0.004 260)", tx2: "oklch(0.900 0.005 260)", tx3: "oklch(0.840 0.006 260)",
  tx4: "oklch(0.785 0.006 260)",
};
const LIGHT_HIGH_COLORS: ColorMap = {
  bd: "oklch(0.470 0.012 260)", bd2: "oklch(0.300 0.014 260)", field: "oklch(0.260 0.014 260)",
  tx: "oklch(0.075 0.006 260)", tx2: "oklch(0.175 0.008 260)", tx3: "oklch(0.250 0.009 260)",
  tx4: "oklch(0.315 0.010 260)",
};
const COLOR_SAFE_COLORS: ColorMap = { ok: "#009e73", warn: "#e69f00", bad: "#d55e00", info: "#0072b2" };
const MONOCHROME_COLORS: ColorMap = {
  ok: "var(--color-tx)", warn: "var(--color-tx2)", bad: "var(--color-tx3)", info: "var(--color-tx2)",
};

function applyColors(root: HTMLElement, profile: ResolvedAccessibilityProfile): void {
  for (const key of COLOR_KEYS) root.style.removeProperty(`--color-${key}`);
  const colors: ColorMap = {};
  if (profile.theme === "light") Object.assign(colors, LIGHT_COLORS);
  if (profile.theme === "calm") Object.assign(colors, CALM_COLORS);
  if (profile.theme === "paper") Object.assign(colors, PAPER_COLORS);
  if (profile.theme === "forced") Object.assign(colors, FORCED_COLORS);
  if (profile.contrast === "high" && profile.theme === "dark") Object.assign(colors, HIGH_COLORS);
  if (profile.contrast === "high" && profile.theme === "light") Object.assign(colors, LIGHT_HIGH_COLORS);
  if (profile.colorVision === "safe") Object.assign(colors, COLOR_SAFE_COLORS);
  if (profile.colorVision === "monochrome") Object.assign(colors, MONOCHROME_COLORS);
  for (const [key, value] of Object.entries(colors)) {
    if (value) root.style.setProperty(`--color-${key}`, value);
  }
}

/**
 * Apply a resolved profile to a root element: the 15 effect `data-*`
 * attributes and the oklch colour tokens. NEEDS-FREE by construction — it
 * writes NO `data-needs` attribute. This is the difference from the reference
 * tool's `applyAccessibilityPreferences`/`preboot.js`, which leak `data-needs`.
 */
export function applyResolvedAxes(
  profile: ResolvedAccessibilityProfile,
  root: HTMLElement,
): void {
  root.setAttribute("data-theme", profile.theme);
  root.setAttribute("data-contrast", profile.contrast);
  root.setAttribute("data-color-vision", profile.colorVision);
  root.setAttribute("data-pointer", profile.pointer);
  root.setAttribute("data-text-spacing", profile.textSpacing);
  root.setAttribute("data-font", profile.font);
  root.setAttribute("data-motion", profile.motion);
  root.setAttribute("data-transparency", profile.transparency);
  root.setAttribute("data-density", profile.density);
  root.setAttribute("data-reading", profile.reading);
  root.setAttribute("data-focus", profile.focus);
  root.setAttribute("data-scale", profile.scale);
  root.setAttribute("data-shortcuts", profile.shortcuts);
  root.setAttribute("data-assistive-tech", profile.assistiveTech);
  root.setAttribute("data-captions", profile.captions);
  applyColors(root, profile);
  root.style.colorScheme = profile.theme === "light" || profile.theme === "paper" ? "light" : "dark";
}

/**
 * Read a resolved profile from a bridge-event payload, validating strictly.
 * Returns `null` if the payload is malformed OR carries any extra key
 * (including a smuggled `needs`). The app calls this on every
 * `A11Y_PROFILE_EVENT` before applying.
 */
export function parseProjectedProfile(payload: unknown): ResolvedAccessibilityProfile | null {
  const parsed = ResolvedAccessibilityProfileSchema.safeParse(payload);
  return parsed.success ? parsed.data : null;
}

/** The shape of the bridge event carrying a profile host→app. */
export interface AccessibilityProfileEvent {
  type: "sbfb-bridge-event";
  name: typeof A11Y_PROFILE_EVENT;
  payload: ResolvedAccessibilityProfile;
}

/**
 * HOST SIDE — build the exact 15-axis bridge event from the operator's
 * preferences. Resolution happens here (needs stay local); the payload is
 * re-validated with the strict schema before it leaves the host, so a bug that
 * ever produced an extra field (a `needs` regression) throws rather than
 * shipping the cause. This is the sender counterpart to the app-side
 * {@link parseProjectedProfile}.
 */
export function buildAccessibilityProfileEvent(
  prefs: AccessibilityPreferences,
  system: AccessibilitySystem = {},
): AccessibilityProfileEvent {
  const resolved = resolveAccessibilityProfile(prefs, system);
  const checked = parseProjectedProfile(resolved);
  if (!checked) {
    throw new Error("resolved accessibility profile failed strict validation — refusing to send");
  }
  return { type: "sbfb-bridge-event", name: A11Y_PROFILE_EVENT, payload: checked };
}

/**
 * HOST SIDE — resolve + push the needs-free profile to a sandboxed app window
 * (the Hub iframe). `needs` never enters the message. Call on iframe load /
 * readiness and whenever the operator's preferences or OS media signals change
 * ("propagée sans reload"). `targetOrigin` defaults to `"*"` because the Hub
 * runs from an OPAQUE sandbox origin (no `allow-same-origin`); the payload
 * carries no secret, only effect axes.
 */
export function pushAccessibilityProfile(
  target: Pick<Window, "postMessage">,
  prefs: AccessibilityPreferences,
  system: AccessibilitySystem = {},
  targetOrigin = "*",
): void {
  target.postMessage(buildAccessibilityProfileEvent(prefs, system), targetOrigin);
}
