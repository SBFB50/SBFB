# Sprint 63 Phase A — preflight G8

Date : 2026-05-15 | HEAD : `b7cc905` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, no band-aid — N/A (MANDATORY carry resolution, not design choice)
- feedback_context7_systematic.md : context7 consulte au kickoff (tray-icon + Playwright) — satisfait

## Scans (all clean)
- S1a OSS prior art : tray icon PNG decode = well-established pattern (tray-icon docs show Icon::from_rgba). Playwright global-setup with custom server = documented pattern (webServer config). APPROACH-ALIGNED — clean
- S1b deps : `png = "0.18.1"` (latest, pure Rust, already transitive via image 0.25.10). image 0.25.10 transitives = bytemuck + byteorder-lite + moxcms + num-traits + png. Swap elimine 5 crates (image + 4 non-png deps). 0 CVE. Note : kickoff mentionnait `png = "0.17"`, corrige en `0.18` (version effective dans le workspace) — clean
- S2 historiques : 4 fichiers scannes (tray.rs, Cargo.toml launcher, global-setup.ts, global-teardown.ts), 0 commit DEVIATION/rejected/scope-cut sur ces fichiers — clean
- S3 threat model : fast-path verified — Phase A ne touche ni composant securite ni wire format. Launcher tray icon + Playwright test infra = hors perimetre threat model — clean
- S4 wire format : fast-path verified — 0 `_VERSION` et 0 `DOMAIN_` dans crates/nexus-launcher/src/. Aucun fichier canonical.rs/schemas touche — clean

## Telemetrie preflight
- Duree totale : ~3min
- S1a : 30s / 2 patterns consultes (tray-icon, Playwright) / APPROACH-ALIGNED
- S1b : 60s / 2 crates scannes (png, image) / clean (version corrigee 0.17→0.18)
- S2 : 30s / 4 fichiers, 0 findings
- S3 : fast-path / 10s
- S4 : fast-path / 10s

## Action
Proceder code Phase A. Version `png` corrigee a `"0.18"` (pas 0.17).
