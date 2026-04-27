# Sprint 33 Phase A — preflight G8

Date : 2026-04-27 | HEAD : `48d0133` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, research before code, context7 obligatoire — research kickoff same-day (2026-04-27), context7 MCP non disponible session, mitige par research fraiche
- feedback_context7_systematic.md : tower-http 0.6 CorsLayer + FastAPI CORSMiddleware couverts par kickoff §Sources context7 (meme jour)

## Scans (all clean)
- S1a OSS prior art : CORS opt-in external origins = pattern standard ecosysteme axum (tower-http CorsLayer AllowOrigin::predicate/list) et FastAPI (CORSMiddleware allow_origins list). APPROACH-ALIGNED — aucun projet OSS ne fait differemment pour du CORS opt-in explicite. Plan conforme SOTA.
- S1b deps : tower-http 0.6.8 (Cargo.lock, publie 2025-12-08 = 140j), fastapi >= 0.111 (pyproject.toml). 0 delta version, 0 CVE identifie (kickoff research same-day) — clean
- S2 historiques : 5 fichiers cibles scannes, 4 commits historiques trouves (S7 Phase A/E http.rs/main.rs/app.py, S30 Phase B COOP/COEP http.rs). S7 audit gate "CORS trust boundary PASS" = decision loopback-only deliberee. Phase A etend avec opt-in explicite `--cors-origin` = evolution, pas contradiction. 0 rejected pattern en conflit. Archive v1.0 mentionne CORS loopback-only comme acceptable (S4/S5/S7/S11/S12). Memory feedback : 0 regle "never" applicable a CORS opt-in — clean
- S3 threat model : fast-path verified. Phase A ne cree ni nouveau composant securite ni nouveau wire format. CORS opt-in preserve loopback-only par defaut (zero regression). HARDENING_ROADMAP last_validated S32 (meme jour), 0 trigger actif S33 — clean
- S4 wire format : fast-path verified. 0 fichier canonical.rs/schemas/ dans perimetre phase. `*_VERSION = 1` preserve (schemas/mod.rs). Day 0 D1-D5 preservees (CORS = implementation D1). Pre-launch policy respectee — clean

## Telemetrie preflight
- Duree totale : ~2m
- S1a : ~30s / 0 projets OSS externes consultes (pattern standard, kickoff research fraiche) / finding : clean (APPROACH-ALIGNED)
- S1b : ~20s / 2 libs scannees (tower-http 0.6.8, fastapi) / finding : clean
- S2 : ~30s / 5 fichiers, 4 commits scannes + archive v1.0 + memory feedback / finding : clean
- S3 : fast-path / ~15s
- S4 : fast-path / ~15s

## Action
Proceder code phase A.
