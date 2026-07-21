// SPDX-License-Identifier: AGPL-3.0-or-later
//! `ResolvedAccessibilityProfile.v1` — the host→app accessibility projection
//! (Sprint 83 Phase B).
//!
//! ## What this contract is
//!
//! An SBFB app (the Workflow Hub) renders inside a sandboxed iframe
//! (`allow-scripts` without `allow-same-origin`, CSP `connect-src 'none'`).
//! The *host* (the React shell / Hub host page) knows the operator's
//! accessibility profile; the app must render accessibly WITHOUT ever being
//! told WHY. This module is the machine-readable identity of the projection
//! the host sends the app: the resolved **effect axes** (`motion=reduced`,
//! `contrast=high`, …) and nothing else.
//!
//! ## The cardinal invariant — `needs[]` never leaves the host
//!
//! The operator's [`AccessibilityNeed`]s (low vision, cognitive, vestibular,
//! …) are health/disability data — a high-sensitivity LINDDUN asset. They
//! drive resolution LOCALLY on the host, then are dropped: the projection
//! carries the EFFECT, never the CAUSE. The invariant holds **by
//! construction**, not by trusting the sandbox — [`ResolvedAccessibility
//! Profile`] has NO `needs` field at all, so the cause cannot leak through the
//! wire JSON, the generated JSON Schema, a `Debug`/`{:?}` print, a logger, or a
//! manual field access. The needs live only on the host-local input
//! [`AccessibilityPreferences`]. [`tests::needs_never_serialized_outward`]
//! guards the projection: re-add a needs field and the test goes red.
//!
//! This is intentionally STRICTER than the reference host tool
//! (`tools/factory-operator/src/preferences/accessibility.ts`), which writes
//! `data-needs` to its own DOM (`applyAccessibilityPreferences`): acceptable
//! for a first-party privileged host tool writing its OWN local DOM (under its
//! own CSP, not the sandbox's `connect-src 'none'`), unacceptable for an
//! untrusted app that could exfiltrate. The resolution logic here mirrors that tool axis
//! for axis so the host and the TS common module agree, but the projection
//! type is needs-free.
//!
//! ## Pre-launch protocol / wire
//!
//! [`ResolvedAccessibilityProfile`] is a host→app projection: LOCAL,
//! non-signed, non-gossiped. It declares NO `DOMAIN_*_V1` and NO
//! `*_FORMAT_VERSION` — inventing one to "tag" the frontier would trip the
//! frozen `DOMAIN_*_V1` census (`scripts/check-frontier-contracts.sh`) and IS
//! a wire bump by definition. Its machine contract is the generated schema
//! snapshot `accessibility_profile.schema.json` next to this module, pinned by
//! [`tests::schema_snapshot`] (the same drift-gate mechanism as
//! `nexus-core-rs` `schemas::shard`), so the identity cannot drift silently.
//! It is deliberately kept OUT of [`crate::contracts::all_contract_schemas`]:
//! it is not one of the seven signed pack/graph/run identities, and adding it
//! there would break that registry's `len == 7` assertion.

use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

/// One accessibility NEED — the operator's declared cause (health/disability
/// data). Mirror of the `NEEDS` union in
/// `tools/factory-operator/src/preferences/accessibility.ts`. These NEVER
/// cross the host→app boundary: they are the input to [`resolve`], not part
/// of the [`ResolvedAccessibilityProfile`] projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum AccessibilityNeed {
    LowVision,
    BlindAssistive,
    ColorVision,
    NoColor,
    Vestibular,
    Photosensitive,
    Motor,
    Cognitive,
    Dyslexia,
    Attention,
    Auditory,
    Speech,
    Sensory,
}

impl AccessibilityNeed {
    /// The 13 needs, in the reference order of `accessibility.ts` `NEEDS`.
    pub const ALL: [AccessibilityNeed; 13] = [
        AccessibilityNeed::LowVision,
        AccessibilityNeed::BlindAssistive,
        AccessibilityNeed::ColorVision,
        AccessibilityNeed::NoColor,
        AccessibilityNeed::Vestibular,
        AccessibilityNeed::Photosensitive,
        AccessibilityNeed::Motor,
        AccessibilityNeed::Cognitive,
        AccessibilityNeed::Dyslexia,
        AccessibilityNeed::Attention,
        AccessibilityNeed::Auditory,
        AccessibilityNeed::Speech,
        AccessibilityNeed::Sensory,
    ];
}

/// Theme the operator asks for (`auto` resolves host-side). Mirror of
/// `THEMES`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ThemePreference {
    Auto,
    Dark,
    Light,
    Calm,
    Paper,
    Forced,
}

/// Resolved theme actually applied (no `auto`). Mirror of `RESOLVED_THEMES`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ResolvedTheme {
    Dark,
    Light,
    Calm,
    Paper,
    Forced,
}

/// Contrast the operator asks for (`auto` resolves host-side).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ContrastPreference {
    Auto,
    Standard,
    High,
}

/// Resolved contrast actually applied (no `auto`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ResolvedContrast {
    Standard,
    High,
}

/// Motion the operator asks for (`system` resolves host-side).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum MotionPreference {
    System,
    Reduced,
}

/// Resolved motion actually applied (no `system`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ResolvedMotion {
    Standard,
    Reduced,
}

/// Colour-vision projection. Mirror of `COLOR_VISIONS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ColorVision {
    Standard,
    Safe,
    Monochrome,
}

/// Pointer sizing. Mirror of `POINTERS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Pointer {
    Standard,
    Large,
}

/// Text spacing. Mirror of `TEXT_SPACINGS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TextSpacing {
    Standard,
    Loose,
}

/// Font choice. Mirror of `FONTS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Font {
    Standard,
    Legible,
}

/// Transparency. Mirror of `TRANSPARENCIES`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Transparency {
    Standard,
    Reduced,
}

/// Layout density. Mirror of `DENSITIES`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Density {
    Standard,
    Focus,
}

/// Reading assistance. Mirror of `READINGS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Reading {
    Standard,
    Assist,
}

/// Focus ring strength. Mirror of `FOCUS_MODES`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Focus {
    Standard,
    Strong,
}

/// UI scale, as a percentage string. Mirror of `SCALES` (`'100'..'150'`).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub enum Scale {
    #[serde(rename = "100")]
    S100,
    #[serde(rename = "112")]
    S112,
    #[serde(rename = "125")]
    S125,
    #[serde(rename = "150")]
    S150,
}

/// Keyboard shortcuts on/off. Mirror of `SHORTCUTS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Shortcuts {
    Off,
    On,
}

/// Assistive-tech mode. Mirror of `ASSISTIVE_TECHS` (`'screen-reader'`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AssistiveTech {
    Standard,
    ScreenReader,
}

/// Captions on/off. Mirror of `CAPTIONS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Captions {
    Off,
    On,
}

/// The operator's base preferences (before need-driven resolution) plus the
/// declared [`AccessibilityNeed`]s. Host-local input to [`resolve`]. Mirror of
/// `AccessibilityPreferences` in the reference host tool. This type may be
/// persisted host-side (like the tool's `localStorage`), but it is NEVER sent
/// to an app — only the needs-free [`ResolvedAccessibilityProfile`] crosses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityPreferences {
    /// Declared needs (the cause). Drives resolution, never projected.
    pub needs: Vec<AccessibilityNeed>,
    pub theme: ThemePreference,
    pub contrast: ContrastPreference,
    pub color_vision: ColorVision,
    pub pointer: Pointer,
    pub text_spacing: TextSpacing,
    pub font: Font,
    pub motion: MotionPreference,
    pub transparency: Transparency,
    pub density: Density,
    pub reading: Reading,
    pub focus: Focus,
    pub scale: Scale,
    pub shortcuts: Shortcuts,
    pub assistive_tech: AssistiveTech,
    pub captions: Captions,
}

impl Default for AccessibilityPreferences {
    /// The reference-tool `DEFAULT_ACCESSIBILITY_PREFERENCES`.
    fn default() -> Self {
        AccessibilityPreferences {
            needs: Vec::new(),
            theme: ThemePreference::Auto,
            contrast: ContrastPreference::Auto,
            color_vision: ColorVision::Standard,
            pointer: Pointer::Standard,
            text_spacing: TextSpacing::Standard,
            font: Font::Standard,
            motion: MotionPreference::System,
            transparency: Transparency::Standard,
            density: Density::Standard,
            reading: Reading::Standard,
            focus: Focus::Standard,
            scale: Scale::S100,
            shortcuts: Shortcuts::Off,
            assistive_tech: AssistiveTech::Standard,
            captions: Captions::Off,
        }
    }
}

/// OS/browser-reported accessibility signals, folded into resolution. Mirror
/// of `AccessibilitySystemPreferences`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilitySystem {
    #[serde(default)]
    pub reduced_motion: bool,
    #[serde(default)]
    pub high_contrast: bool,
    #[serde(default)]
    pub forced_colors: bool,
}

/// `ResolvedAccessibilityProfile.v1` — the needs-free host→app projection.
///
/// Carries ONLY the 15 resolved EFFECT axes. There is deliberately NO `needs`
/// field: the cause is structurally absent, so it cannot leak through the wire,
/// the JSON Schema, a `Debug` print, a logger, or a manual field access. The
/// needs live only in [`AccessibilityPreferences`] (the host-local input to
/// [`resolve`]); the output type simply never carries them. This is stronger
/// than a `#[serde(skip)]` on a retained field (which `Debug`/`{:?}` would
/// still print). See the module docs for the cardinal invariant.
///
/// The absence is structurally enforced — the field cannot even be named:
///
/// ```compile_fail
/// use sbfb_workflow_core::accessibility_profile::{
///     resolve, AccessibilityPreferences, AccessibilitySystem,
/// };
/// let p = resolve(&AccessibilityPreferences::default(), &AccessibilitySystem::default());
/// let _cause = p.needs; // no field `needs` on ResolvedAccessibilityProfile
/// ```
///
// Host→app accessibility-projection frontier (Sprint 83 Phase B, doctrine §7):
// consumed by a distinct runtime — the Workflow Hub app inside a sandboxed
// iframe. Not a signed wire type, so it carries no `// FRONTIER:`
// domain/version tag (that opt-in registry is for `DOMAIN_*_V1` families); its
// machine contract is the generated schema snapshot below
// (`accessibility_profile.schema.json`, drift-gated) plus the S83 docs-contract
// closure index (GUIDE deferred to the wrap-up phase per AM-3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[schemars(title = "ResolvedAccessibilityProfile.v1")]
pub struct ResolvedAccessibilityProfile {
    pub theme: ResolvedTheme,
    pub contrast: ResolvedContrast,
    pub color_vision: ColorVision,
    pub pointer: Pointer,
    pub text_spacing: TextSpacing,
    pub font: Font,
    pub motion: ResolvedMotion,
    pub transparency: Transparency,
    pub density: Density,
    pub reading: Reading,
    pub focus: Focus,
    pub scale: Scale,
    pub shortcuts: Shortcuts,
    pub assistive_tech: AssistiveTech,
    pub captions: Captions,
}

fn has_need(needs: &[AccessibilityNeed], need: AccessibilityNeed) -> bool {
    needs.contains(&need)
}

fn has_any_need(needs: &[AccessibilityNeed], any: &[AccessibilityNeed]) -> bool {
    any.iter().any(|n| needs.contains(n))
}

fn at_least_scale(current: Scale, minimum: Scale) -> Scale {
    if current >= minimum { current } else { minimum }
}

fn resolve_theme(prefs: &AccessibilityPreferences, system: &AccessibilitySystem) -> ResolvedTheme {
    match prefs.theme {
        ThemePreference::Auto => {}
        ThemePreference::Dark => return ResolvedTheme::Dark,
        ThemePreference::Light => return ResolvedTheme::Light,
        ThemePreference::Calm => return ResolvedTheme::Calm,
        ThemePreference::Paper => return ResolvedTheme::Paper,
        ThemePreference::Forced => return ResolvedTheme::Forced,
    }
    if system.forced_colors
        || has_any_need(
            &prefs.needs,
            &[
                AccessibilityNeed::LowVision,
                AccessibilityNeed::BlindAssistive,
                AccessibilityNeed::NoColor,
            ],
        )
    {
        return ResolvedTheme::Forced;
    }
    if has_any_need(
        &prefs.needs,
        &[
            AccessibilityNeed::Vestibular,
            AccessibilityNeed::Photosensitive,
            AccessibilityNeed::Attention,
            AccessibilityNeed::Sensory,
        ],
    ) {
        return ResolvedTheme::Calm;
    }
    if has_any_need(
        &prefs.needs,
        &[AccessibilityNeed::Cognitive, AccessibilityNeed::Dyslexia],
    ) {
        return ResolvedTheme::Paper;
    }
    ResolvedTheme::Dark
}

fn resolve_contrast(
    prefs: &AccessibilityPreferences,
    system: &AccessibilitySystem,
) -> ResolvedContrast {
    if matches!(prefs.contrast, ContrastPreference::High)
        || system.high_contrast
        || system.forced_colors
        || has_any_need(
            &prefs.needs,
            &[
                AccessibilityNeed::LowVision,
                AccessibilityNeed::BlindAssistive,
                AccessibilityNeed::ColorVision,
                AccessibilityNeed::NoColor,
            ],
        )
    {
        ResolvedContrast::High
    } else {
        ResolvedContrast::Standard
    }
}

/// Resolve a needs-free [`ResolvedAccessibilityProfile`] from the operator's
/// preferences and the OS/browser signals.
///
/// Faithful axis-for-axis port of `resolveAccessibilityPreferences` in
/// `tools/factory-operator/src/preferences/accessibility.ts`, so the Rust
/// contract and the TS host module agree. The stacking is order-sensitive and
/// deliberately reproduced verbatim.
pub fn resolve(
    prefs: &AccessibilityPreferences,
    system: &AccessibilitySystem,
) -> ResolvedAccessibilityProfile {
    let mut scale = prefs.scale;
    let mut pointer = prefs.pointer;
    let mut text_spacing = prefs.text_spacing;
    let mut font = prefs.font;
    let mut color_vision = prefs.color_vision;
    let mut transparency = prefs.transparency;
    let mut density = prefs.density;
    let mut reading = prefs.reading;
    let mut focus = prefs.focus;
    let mut shortcuts = prefs.shortcuts;
    let mut assistive_tech = prefs.assistive_tech;
    let mut captions = prefs.captions;
    let mut motion = if matches!(prefs.motion, MotionPreference::Reduced) || system.reduced_motion {
        ResolvedMotion::Reduced
    } else {
        ResolvedMotion::Standard
    };

    if has_need(&prefs.needs, AccessibilityNeed::LowVision) {
        scale = at_least_scale(scale, Scale::S125);
        pointer = Pointer::Large;
        focus = Focus::Strong;
    }
    if has_need(&prefs.needs, AccessibilityNeed::BlindAssistive) {
        assistive_tech = AssistiveTech::ScreenReader;
        density = Density::Focus;
        focus = Focus::Strong;
        shortcuts = Shortcuts::Off;
    }
    if has_need(&prefs.needs, AccessibilityNeed::ColorVision) {
        color_vision = if matches!(color_vision, ColorVision::Monochrome) {
            ColorVision::Monochrome
        } else {
            ColorVision::Safe
        };
        focus = Focus::Strong;
    }
    if has_need(&prefs.needs, AccessibilityNeed::NoColor) {
        color_vision = ColorVision::Monochrome;
        focus = Focus::Strong;
    }
    if has_any_need(
        &prefs.needs,
        &[
            AccessibilityNeed::Vestibular,
            AccessibilityNeed::Photosensitive,
            AccessibilityNeed::Attention,
            AccessibilityNeed::Sensory,
        ],
    ) {
        motion = ResolvedMotion::Reduced;
        transparency = Transparency::Reduced;
    }
    if has_need(&prefs.needs, AccessibilityNeed::Motor) {
        pointer = Pointer::Large;
        focus = Focus::Strong;
        shortcuts = Shortcuts::Off;
    }
    if has_need(&prefs.needs, AccessibilityNeed::Cognitive) {
        density = Density::Focus;
        reading = Reading::Assist;
        text_spacing = TextSpacing::Loose;
        shortcuts = Shortcuts::Off;
    }
    if has_need(&prefs.needs, AccessibilityNeed::Dyslexia) {
        font = Font::Legible;
        text_spacing = TextSpacing::Loose;
        reading = Reading::Assist;
        scale = at_least_scale(scale, Scale::S112);
    }
    if has_need(&prefs.needs, AccessibilityNeed::Auditory) {
        captions = Captions::On;
    }
    if has_need(&prefs.needs, AccessibilityNeed::Speech) {
        shortcuts = Shortcuts::Off;
    }

    ResolvedAccessibilityProfile {
        theme: resolve_theme(prefs, system),
        contrast: resolve_contrast(prefs, system),
        color_vision,
        pointer,
        text_spacing,
        font,
        motion,
        transparency,
        density,
        reading,
        focus,
        scale,
        shortcuts,
        assistive_tech,
        captions,
    }
}

/// The generated JSON Schema (draft 2020-12) for
/// [`ResolvedAccessibilityProfile`] — the machine contract handed to the app
/// side. `needs` is absent by construction (the type has no such field).
pub fn resolved_profile_schema() -> serde_json::Value {
    serde_json::to_value(schema_for!(ResolvedAccessibilityProfile))
        .expect("ResolvedAccessibilityProfile schema serializes")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The 15 effect axes that make up the projection — the EXACT top-level
    /// key set of both the serialized profile and its schema `properties`.
    /// `colorVision` is a legitimate axis KEY here (it is also a
    /// [`AccessibilityNeed`] name — a deliberate collision inherited from the
    /// reference tool); the leak tests distinguish key from value.
    const EFFECT_AXES: [&str; 15] = [
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
    ];

    fn sorted(mut v: Vec<&str>) -> Vec<&str> {
        v.sort_unstable();
        v
    }

    fn need_names() -> Vec<String> {
        AccessibilityNeed::ALL
            .iter()
            .map(|n| {
                serde_json::to_value(n)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string()
            })
            .collect()
    }

    /// A profile derived from EVERY declared need — the strongest input for
    /// the leak test.
    fn profile_with_all_needs() -> ResolvedAccessibilityProfile {
        let prefs = AccessibilityPreferences {
            needs: AccessibilityNeed::ALL.to_vec(),
            ..AccessibilityPreferences::default()
        };
        resolve(&prefs, &AccessibilitySystem::default())
    }

    /// CARDINAL INVARIANT: `needs[]` never crosses the host→app boundary.
    /// Resolving a profile from all 13 needs must yield a projection that
    /// (1) omits the `needs` key, (2) carries EXACTLY the 15 effect axes, and
    /// (3) never carries a need name as a VALUE (the axis KEY `colorVision` is
    /// part of the contract and reveals nothing — a need as a value would be a
    /// cause leak). Since `ResolvedAccessibilityProfile` has NO `needs` field,
    /// (1)/(2) hold structurally; re-adding a needs field WITHOUT
    /// `#[serde(skip)]` makes assertion (2) go red. A *skipped* re-add would
    /// still slip past serialization — that regression is caught instead by
    /// [`debug_output_carries_no_need`] (the `Debug`/logger vector) and by the
    /// `compile_fail` doctest on [`ResolvedAccessibilityProfile`] proving the
    /// field cannot even be named.
    #[test]
    fn needs_never_serialized_outward() {
        let profile = profile_with_all_needs();
        let value = serde_json::to_value(&profile).expect("profile serializes");
        let obj = value.as_object().expect("profile is a JSON object");

        // (1) the `needs` array field is absent.
        assert!(
            !obj.contains_key("needs"),
            "the `needs` key leaked into the wire projection: {value}"
        );
        // (2) exactly the 15 effect axes — no cause field under another name.
        assert_eq!(
            sorted(obj.keys().map(String::as_str).collect()),
            sorted(EFFECT_AXES.to_vec()),
            "the projection must carry exactly the 15 effect axes"
        );
        // (3) no need name appears as a VALUE.
        let needs = need_names();
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                assert!(
                    !needs.iter().any(|n| n == s),
                    "need `{s}` (a cause) leaked as the value of `{k}`: {value}"
                );
            }
        }
    }

    /// The `Debug`/logger vector (Codex S83-B P1): a `#[serde(skip)]` field
    /// would slip past serialization but STILL print via `{:?}`. The projection
    /// type has no `needs` field, so the `Debug` output of a profile built from
    /// all 13 needs carries no need identifier and no `needs` field name.
    /// Re-adding a `needs` field (skipped or not) makes this go red.
    #[test]
    fn debug_output_carries_no_need() {
        let dbg = format!("{:?}", profile_with_all_needs()).to_lowercase();
        assert!(
            !dbg.contains("needs"),
            "Debug output names a `needs` field: {dbg}"
        );
        for need in need_names() {
            assert!(
                !dbg.contains(&need.to_lowercase()),
                "need `{need}` leaked into the Debug output: {dbg}"
            );
        }
    }

    /// The generated JSON Schema must expose EXACTLY the 15 effect axes and no
    /// `needs` property (an app consuming the schema must not even learn the
    /// field exists). Holds because the type has no `needs` field. The schema
    /// *description* may mention needs in prose — that documents the invariant
    /// and is not a property.
    #[test]
    fn schema_has_no_needs_property() {
        let schema = resolved_profile_schema();
        let props = schema
            .get("properties")
            .and_then(|v| v.as_object())
            .expect("schema exposes properties");
        assert!(
            !props.contains_key("needs"),
            "the generated schema must not expose a `needs` property"
        );
        assert_eq!(
            sorted(props.keys().map(String::as_str).collect()),
            sorted(EFFECT_AXES.to_vec()),
            "the schema must expose exactly the 15 effect axes"
        );
        assert_eq!(
            schema.get("title").and_then(|v| v.as_str()),
            Some("ResolvedAccessibilityProfile.v1"),
            "schema title must equal the contract identity"
        );
    }

    /// The projection carries only the effect (`motion=reduced`), never the
    /// cause (`vestibular`). A vestibular-only operator gets reduced motion +
    /// reduced transparency + calm theme, and the JSON says only that.
    #[test]
    fn projects_effect_not_cause() {
        let prefs = AccessibilityPreferences {
            needs: vec![AccessibilityNeed::Vestibular],
            ..AccessibilityPreferences::default()
        };
        let profile = resolve(&prefs, &AccessibilitySystem::default());
        assert_eq!(profile.motion, ResolvedMotion::Reduced);
        assert_eq!(profile.transparency, Transparency::Reduced);
        assert_eq!(profile.theme, ResolvedTheme::Calm);
        let json = serde_json::to_string(&profile).expect("serializes");
        assert!(json.contains("\"motion\":\"reduced\""), "{json}");
        assert!(!json.contains("vestibular"), "cause leaked: {json}");
    }

    /// Faithful-port spot checks against `accessibility.ts`
    /// `resolveAccessibilityPreferences`.
    #[test]
    fn resolution_matches_reference_axes() {
        // low vision: scale >= 125, large pointer, strong focus, forced theme,
        // high contrast.
        let low = resolve(
            &AccessibilityPreferences {
                needs: vec![AccessibilityNeed::LowVision],
                ..Default::default()
            },
            &AccessibilitySystem::default(),
        );
        assert_eq!(low.scale, Scale::S125);
        assert_eq!(low.pointer, Pointer::Large);
        assert_eq!(low.focus, Focus::Strong);
        assert_eq!(low.theme, ResolvedTheme::Forced);
        assert_eq!(low.contrast, ResolvedContrast::High);

        // dyslexia: legible font, loose spacing, assist reading, scale >= 112,
        // paper theme.
        let dys = resolve(
            &AccessibilityPreferences {
                needs: vec![AccessibilityNeed::Dyslexia],
                ..Default::default()
            },
            &AccessibilitySystem::default(),
        );
        assert_eq!(dys.font, Font::Legible);
        assert_eq!(dys.text_spacing, TextSpacing::Loose);
        assert_eq!(dys.reading, Reading::Assist);
        assert_eq!(dys.scale, Scale::S112);
        assert_eq!(dys.theme, ResolvedTheme::Paper);

        // blind + assistive: screen reader, focus density, shortcuts off.
        let blind = resolve(
            &AccessibilityPreferences {
                needs: vec![AccessibilityNeed::BlindAssistive],
                ..Default::default()
            },
            &AccessibilitySystem::default(),
        );
        assert_eq!(blind.assistive_tech, AssistiveTech::ScreenReader);
        assert_eq!(blind.density, Density::Focus);
        assert_eq!(blind.shortcuts, Shortcuts::Off);

        // system reduced-motion folds in even with no needs.
        let sys = resolve(
            &AccessibilityPreferences::default(),
            &AccessibilitySystem {
                reduced_motion: true,
                ..Default::default()
            },
        );
        assert_eq!(sys.motion, ResolvedMotion::Reduced);

        // no needs, no system signals: everything standard, dark theme.
        let base = resolve(
            &AccessibilityPreferences::default(),
            &AccessibilitySystem::default(),
        );
        assert_eq!(base.theme, ResolvedTheme::Dark);
        assert_eq!(base.contrast, ResolvedContrast::Standard);
        assert_eq!(base.motion, ResolvedMotion::Standard);
        assert_eq!(base.scale, Scale::S100);
    }

    /// Scale value serialisation must match the TS string union (`'100'`),
    /// not the Rust variant name — the app reads the same tokens.
    #[test]
    fn scale_serializes_as_percentage_string() {
        assert_eq!(serde_json::to_string(&Scale::S100).unwrap(), "\"100\"");
        assert_eq!(serde_json::to_string(&Scale::S150).unwrap(), "\"150\"");
    }

    /// Golden snapshot of the projection schema against
    /// `accessibility_profile.schema.json` next to this module — the canary
    /// against silent identity drift once axes change. Refresh an INTENTIONAL
    /// change with
    /// `UPDATE_SNAPSHOTS=1 cargo test -p sbfb-workflow-core schema_snapshot`.
    ///
    /// NOT fail-open (Codex S83-B): the snapshot self-writes ONLY under
    /// `UPDATE_SNAPSHOTS`. A missing or empty committed golden FAILS the test
    /// (asking to regenerate + commit) instead of silently going green — the
    /// discipline `contracts::tests::schema_snapshot_stub` adopted in turn in
    /// Phase F (both are strict now; the §P75 bootstrap self-write is gone
    /// from this crate).
    #[test]
    fn schema_snapshot() {
        let live = resolved_profile_schema();
        let live_pretty = serde_json::to_string_pretty(&live).unwrap();

        let snapshot_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("accessibility_profile.schema.json");

        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            std::fs::write(&snapshot_path, format!("{live_pretty}\n"))
                .expect("write accessibility_profile.schema.json snapshot");
            return;
        }

        let snapshot_str = std::fs::read_to_string(&snapshot_path).expect(
            "accessibility_profile.schema.json missing — regenerate with \
             UPDATE_SNAPSHOTS=1 and commit it",
        );
        assert!(
            !snapshot_str.trim().is_empty(),
            "accessibility_profile.schema.json is empty — regenerate with \
             UPDATE_SNAPSHOTS=1 and commit it"
        );
        let snapshot: serde_json::Value =
            serde_json::from_str(&snapshot_str).expect("snapshot must be valid JSON");

        assert_eq!(
            live, snapshot,
            "accessibility profile schema drift — run with UPDATE_SNAPSHOTS=1 to refresh"
        );
    }

    /// Cross-implementation parity oracle (Codex S83-B P2). A committed fixture
    /// (`accessibility_fixtures.json`) maps `needs + system` inputs to the exact
    /// 15-axis projection; BOTH this test and the TS
    /// `profile_resolution.test.ts` consume it. A divergence in EITHER `resolve`
    /// implementation (or a hand-edit of the fixture) fails against the shared
    /// oracle. NOT fail-open: a missing fixture FAILS (asking to regenerate),
    /// never self-passes.
    ///
    /// Regenerate an intentional change with
    /// `UPDATE_SNAPSHOTS=1 cargo test -p sbfb-workflow-core resolution_fixtures`.
    #[test]
    fn resolution_fixtures() {
        // (name, needs, system) — each isolated need, the OS signals, and a few
        // stacking combinations that exercise priority order.
        let sys = |rm: bool, hc: bool, fc: bool| AccessibilitySystem {
            reduced_motion: rm,
            high_contrast: hc,
            forced_colors: fc,
        };
        let cases: Vec<(&str, Vec<AccessibilityNeed>, AccessibilitySystem)> = vec![
            ("baseline", vec![], AccessibilitySystem::default()),
            (
                "lowVision",
                vec![AccessibilityNeed::LowVision],
                AccessibilitySystem::default(),
            ),
            (
                "blindAssistive",
                vec![AccessibilityNeed::BlindAssistive],
                AccessibilitySystem::default(),
            ),
            (
                "colorVision",
                vec![AccessibilityNeed::ColorVision],
                AccessibilitySystem::default(),
            ),
            (
                "noColor",
                vec![AccessibilityNeed::NoColor],
                AccessibilitySystem::default(),
            ),
            (
                "vestibular",
                vec![AccessibilityNeed::Vestibular],
                AccessibilitySystem::default(),
            ),
            (
                "photosensitive",
                vec![AccessibilityNeed::Photosensitive],
                AccessibilitySystem::default(),
            ),
            (
                "motor",
                vec![AccessibilityNeed::Motor],
                AccessibilitySystem::default(),
            ),
            (
                "cognitive",
                vec![AccessibilityNeed::Cognitive],
                AccessibilitySystem::default(),
            ),
            (
                "dyslexia",
                vec![AccessibilityNeed::Dyslexia],
                AccessibilitySystem::default(),
            ),
            (
                "attention",
                vec![AccessibilityNeed::Attention],
                AccessibilitySystem::default(),
            ),
            (
                "auditory",
                vec![AccessibilityNeed::Auditory],
                AccessibilitySystem::default(),
            ),
            (
                "speech",
                vec![AccessibilityNeed::Speech],
                AccessibilitySystem::default(),
            ),
            (
                "sensory",
                vec![AccessibilityNeed::Sensory],
                AccessibilitySystem::default(),
            ),
            ("system-reduced-motion", vec![], sys(true, false, false)),
            ("system-high-contrast", vec![], sys(false, true, false)),
            ("system-forced-colors", vec![], sys(false, false, true)),
            (
                "stack-lowVision-dyslexia",
                vec![AccessibilityNeed::LowVision, AccessibilityNeed::Dyslexia],
                AccessibilitySystem::default(),
            ),
            (
                "stack-cognitive-motor",
                vec![AccessibilityNeed::Cognitive, AccessibilityNeed::Motor],
                AccessibilitySystem::default(),
            ),
            (
                "all-needs",
                AccessibilityNeed::ALL.to_vec(),
                AccessibilitySystem::default(),
            ),
        ];

        // Explicit-preference cases (Codex S83-B P2): exercise output tokens the
        // need-driven cases never emit — light theme, scale 150, shortcuts on,
        // high-contrast preference, and a NON-degrading scale (a low need floor
        // must not lower an already-high explicit scale).
        let pref_cases: Vec<(&str, AccessibilityPreferences, AccessibilitySystem)> = vec![
            (
                "pref-theme-light",
                AccessibilityPreferences {
                    theme: ThemePreference::Light,
                    ..AccessibilityPreferences::default()
                },
                AccessibilitySystem::default(),
            ),
            (
                "pref-scale-150-shortcuts-on",
                AccessibilityPreferences {
                    scale: Scale::S150,
                    shortcuts: Shortcuts::On,
                    ..AccessibilityPreferences::default()
                },
                AccessibilitySystem::default(),
            ),
            (
                "pref-contrast-high-motion-reduced",
                AccessibilityPreferences {
                    contrast: ContrastPreference::High,
                    motion: MotionPreference::Reduced,
                    ..AccessibilityPreferences::default()
                },
                AccessibilitySystem::default(),
            ),
            (
                "pref-nondegrading-scale-150-with-dyslexia",
                AccessibilityPreferences {
                    scale: Scale::S150,
                    needs: vec![AccessibilityNeed::Dyslexia],
                    ..AccessibilityPreferences::default()
                },
                AccessibilitySystem::default(),
            ),
        ];

        let all_cases = cases
            .iter()
            .map(|(name, needs, system)| {
                (
                    *name,
                    AccessibilityPreferences {
                        needs: needs.clone(),
                        ..AccessibilityPreferences::default()
                    },
                    *system,
                )
            })
            .chain(pref_cases.iter().map(|(n, p, s)| (*n, p.clone(), *s)));

        let computed: serde_json::Value = serde_json::json!({
            "note": "Sprint 83 Phase B — shared Rust<->TS resolution parity oracle. \
                     full prefs + system -> the 15-axis needs-free projection. Consumed by \
                     accessibility_profile.rs and web/src/a11y/__tests__/profile_resolution.test.ts. \
                     Regenerate: UPDATE_SNAPSHOTS=1 cargo test -p sbfb-workflow-core resolution_fixtures.",
            "cases": all_cases.map(|(name, prefs, system)| {
                serde_json::json!({
                    "name": name,
                    "prefs": serde_json::to_value(&prefs).unwrap(),
                    "system": serde_json::to_value(system).unwrap(),
                    "expected": serde_json::to_value(resolve(&prefs, &system)).unwrap(),
                })
            }).collect::<Vec<_>>(),
        });

        let fixtures_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("accessibility_fixtures.json");

        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            std::fs::write(
                &fixtures_path,
                format!("{}\n", serde_json::to_string_pretty(&computed).unwrap()),
            )
            .expect("write accessibility_fixtures.json");
            return;
        }

        let committed_str = std::fs::read_to_string(&fixtures_path).expect(
            "accessibility_fixtures.json missing — regenerate with \
             UPDATE_SNAPSHOTS=1 and commit it",
        );
        let committed: serde_json::Value =
            serde_json::from_str(&committed_str).expect("fixtures must be valid JSON");
        assert_eq!(
            computed, committed,
            "resolution fixtures drift — Rust resolve() diverged from the committed \
             oracle (or the fixture was hand-edited); run with UPDATE_SNAPSHOTS=1 to refresh"
        );
    }
}
