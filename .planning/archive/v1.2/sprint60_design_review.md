# Sprint 60 — Design Review (G1)

**Date** : 2026-05-11
**Reviewer** : agent Explore independant (session fraiche)
**Sprint** : 60 (installer Windows + tray icon + frontend bundling
→ tag v1.0)

---

## Scoring

| Decision | Score | Justification |
|----------|-------|---------------|
| D1 — cargo-packager + NSIS | ✅ | cargo-packager v0.11.8 confirme (crates.io). NSIS 3.12 stable (19 avril 2026). Multi-binaires documente. WiX v7 vs cargo-wix v3 gap maintenance valide. |
| D2 — tray-icon (Tauri) | ⚠️ | tray-icon v0.24.0 (crates.io latest), Tauri team actif. Message pump Windows standalone sans winit non cross-check dans la doc officielle. Pas de GH issues bloquants recents. muda v0.19.1 confirme. |
| D3 — Frontend bundling disk | ✅ | rust-embed correctement rejete (+5-10MB binaire, mismatch update). --web-root + ServeDir existant et documente dans le code. |
| D4 — LT-7 Tier 3 quorum | ⚠️ | Rust reproducible builds cross-OS bloques en 2026 (rust-lang/rust#129080 non resolu, MSVC random seed). Quorum intra-OS valide pour MVP. |
| D5 — Scope change frontend P2P → bundling | ✅ | Changement explicitement documente avec rationale. Pas de drift silencieux. CLAUDE.md sera synchronise en Phase E. |

---

## Verdict : PASS

2 ⚠️ (D2 message pump, D4 reproducible builds) acknowledges dans
le kickoff §4 avec fallbacks documentes (R2 windows-sys direct,
R3 consensus intra-OS). 0 ❌. Les ⚠️ sont des risques techniques
mitiges, pas des choix contredits par les sources. Les D1/D3/D5
sont solidement fondes. Le sprint peut proceder avec les D1-D5
gelees.

---

## Alternatives non-citees detectees

Aucune alternative majeure non-citee detectee. Les 5 decisions
couvrent les options pertinentes 2026 avec sources verifiees par
`cargo search` (versions crates.io 2026-05-11).

## Checklist crypto/spec

Non applicable — S60 ne touche pas de composant crypto ni de spec
standardisee.

## Checklist Rust-first

Toutes les decisions utilisent des crates Rust (cargo-packager,
tray-icon, muda). Pas de runtime non-Rust introduit.

---

## Sources

- `cargo search cargo-packager` → 0.11.8 (confirme)
- `cargo search tray-icon` → 0.24.0 (corrige de 0.21.3)
- `cargo search muda` → 0.19.1 (corrige de 0.17)
- `cargo search iroh` → 1.0.0-rc.0 (trigger G2 ACTIF)
- NSIS 3.12 : https://nsis.sourceforge.io/Download (corrige de 3.11)
- Rust reproducible builds : reproducible-builds.org/reports/2026-04/
- docs/architecture/SELF_HOSTED_BUILD.md §5 §10
