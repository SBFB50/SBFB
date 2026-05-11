# Sprint 60 — Design Review (G1)

**Date** : 2026-05-11
**Reviewer** : agent Explore independant (session fraiche)
**Sprint** : 60 (installer Windows + tray icon + frontend bundling
→ tag v1.0)

---

## Scoring

| Decision | Score | Justification |
|----------|-------|---------------|
| D1 — cargo-packager + NSIS | ✅ | cargo-packager v0.11.8 actif (mars 2026 commit), NSIS 3.11 stable (avril 2026, CVE-2025-43715 patche). Multi-binaires documente. WiX v7 vs cargo-wix v3 gap maintenance valide. |
| D2 — tray-icon (Tauri) | ⚠️ | tray-icon v0.21.3 actif, Tauri team. Message pump Windows standalone sans winit non cross-check dans la doc. Pas de GH issues bloquants recents. |
| D3 — Frontend bundling disk | ✅ | rust-embed correctement rejete (+5-10MB binaire, mismatch update). --web-root + ServeDir existant et documente dans le code. |
| D4 — LT-7 Tier 3 quorum | ⚠️ | Rust reproducible builds cross-OS bloques en 2026 (rust-lang/rust#129080 non resolu, MSVC random seed). Quorum intra-OS valide pour MVP. |

---

## Alternatives non-citees detectees

Aucune alternative majeure non-citee detectee. Les 4 decisions
couvrent les options pertinentes 2026 avec sources datees < 90j.

## Checklist crypto/spec

Non applicable — S60 ne touche pas de composant crypto ni de spec
standardisee.

## Checklist Rust-first

Toutes les decisions utilisent des crates Rust (cargo-packager,
tray-icon, muda). Pas de runtime non-Rust introduit.

---

## Sources

- CrabNebula GitHub (cargo-packager activity mars 2026)
- Tauri tray-icon releases (v0.21.3 jan 2026)
- Rust reproducible builds (reproducible-builds.org/reports/2026-04/)
- docs/architecture/SELF_HOSTED_BUILD.md §5 §10
